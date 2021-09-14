mod ad4m;
mod test_inspect;

use hdk::prelude::*;
use ad4m::PerspectiveExpression;
use test_inspect::Recipient;
use test_inspect::get_test_recipient;

#[hdk_entry(id = "message_wrapper", visibility = "private")]
#[derive(Clone)]
enum MessageWrapper {
    StatusRequest,
    StatusResponse(PerspectiveExpression),
    DirectMessage(PerspectiveExpression)
}

#[hdk_entry(id = "status_update", visibility = "private")]
#[derive(Clone)]
pub struct StatusUpdate(PerspectiveExpression);

impl Into<PerspectiveExpression> for StatusUpdate {
    fn into(self) -> PerspectiveExpression {
        self.0
    }
}

#[hdk_entry(id = "message", visibility = "private")]
#[derive(Clone)]
pub struct StoredMessage(PerspectiveExpression);

impl Into<PerspectiveExpression> for StoredMessage {
    fn into(self) -> PerspectiveExpression {
        self.0
    }
}

#[hdk_entry(id = "message", visibility = "public")]
#[derive(Clone)]
pub struct PublicMessage(PerspectiveExpression);

impl Into<PerspectiveExpression> for PublicMessage {
    fn into(self) -> PerspectiveExpression {
        self.0
    }
}

#[derive(Serialize, Deserialize, Debug, SerializedBytes)]
pub struct Properties {
    pub recipient_hc_agent_pubkey: AgentPubKey,
}

#[derive(Serialize, Deserialize, Debug, SerializedBytes)]
pub struct Signal {
    pub json: String,
}


entry_defs![
    Path::entry_def(),
    PerspectiveExpression::entry_def(),
    Recipient::entry_def(),
    StatusUpdate::entry_def(),
    StoredMessage::entry_def()
];

#[hdk_extern]
fn init(_: ()) -> ExternResult<InitCallbackResult> {
    debug!("INIT called");
    let mut functions: GrantedFunctions = BTreeSet::new();
    functions.insert((zome_info()?.zome_name, "recv_remote_signal".into()));
    functions.insert((zome_info()?.zome_name, "get_status".into()));

    //Create open cap grant to allow agents to send signals of links to each other
    create_cap_grant(CapGrantEntry {
        tag: "".into(),
        // empty access converts to unrestricted
        access: ().into(),
        functions,
    })?;
    Ok(InitCallbackResult::Pass)
}

fn recipient() -> ExternResult<AgentPubKey> {
    if let Some(recipient) = get_test_recipient(())? {
        Ok(recipient.get())
    } else {
        let properties = Properties::try_from(zome_info()?.properties)?;
        Ok(properties.recipient_hc_agent_pubkey)
    }
}

//---------------------------------------------------------
//----Status-----------------------------------------------
//---------------------------------------------------------

#[hdk_extern]
pub fn set_status(new_status: PerspectiveExpression) -> ExternResult<()> {
    if agent_info()?.agent_latest_pubkey == recipient()? {
        create_entry(StatusUpdate(new_status))?;
        Ok(())
    } else {
        Err(WasmError::Guest(String::from("Only recipient can set their status")))
    }
}

#[hdk_extern]
pub fn get_status(_: ()) -> ExternResult<Option<PerspectiveExpression>> {
    if agent_info()?.agent_latest_pubkey == recipient()? {
        // If called on the recipient node 
        // (either from local ad4m-executor or via remote_call)
        // we retrieve the latest status entry from source chain
        let mut filter = QueryFilter::new();
        filter.entry_type = Some(entry_type!(StatusUpdate)?);
        filter.include_entries = true;
        if let Some(element) = query(filter)?.pop() {
            let status = StatusUpdate::try_from(element)?;
            Ok(Some(status.into()))
        } else {
            Ok(None)
        }
    } else {
        // Otherwise proxy to recipient
        match call_remote(recipient()?, zome_info()?.zome_name, "get_status".into(), None, ())? {
            ZomeCallResponse::Ok(extern_io) => {
                Ok(extern_io.decode()?)
            },
            ZomeCallResponse::Unauthorized(_,_,_,_) => Err(WasmError::Guest(String::from("Unauthorized error"))),
            ZomeCallResponse::NetworkError(error) => Err(WasmError::Guest(error)),
            ZomeCallResponse::CountersigningSession(session) => Err(WasmError::Guest(session)),
        }
    }
}


//---------------------------------------------------------
//----Messages---------------------------------------------
//---------------------------------------------------------


#[hdk_extern]
fn recv_remote_signal(signal: SerializedBytes) -> ExternResult<()> {
    debug!("RECEIVEING MESSAGE...");
    match PerspectiveExpression::try_from(signal) {
        Ok(message) => {
            let json = serde_json::to_string(&message).unwrap();
            emit_signal(&SerializedBytes::try_from(Signal{json})?)?;
            create_entry(StoredMessage(message))?;
            Ok(())
        },
        Err(error) => {
            let error_message = format!("Received signal that does not parse to PerspectiveExpression: {}", error);
            debug!("Error in recv_remote_sigal: {}", error_message);
            Err(WasmError::Guest(String::from(error_message)))
        }
    }
}

#[hdk_extern]
fn inbox(_: ()) -> ExternResult<Vec<PerspectiveExpression>> {
    let mut filter = QueryFilter::new();
    filter.entry_type = Some(entry_type!(StoredMessage)?);
    filter.include_entries = true;
    Ok(query(filter)?
        .iter()
        .filter_map(|m| PerspectiveExpression::try_from(m).ok())
        .collect()
    )
}



#[hdk_extern]
pub fn send_p2p(message: PerspectiveExpression) -> ExternResult<()> {
    debug!("SENDING MESSAGE...");
    remote_signal(SerializedBytes::try_from(message)?, vec![recipient()?])
}

fn inbox_hash() -> ExternResult<EntryHash> {
    let path = Path::from("inbox");
    path.ensure()?;
    Ok(path.hash()?)
}

#[hdk_extern]
pub fn send_inbox(message: PerspectiveExpression) -> ExternResult<()> {
    let entry = PublicMessage(message);
    let entry_hash = hash_entry(&entry)?;
    create_entry(entry)?;
    create_link(inbox_hash()?, entry_hash, ())?;
    debug!("Link created");
    debug!("inbox_hash: {}", inbox_hash()?);
    Ok(())
}

#[hdk_extern]
pub fn fetch_inbox(_: ()) -> ExternResult<()> {
    debug!("fetch_inbox");
    if agent_info()?.agent_latest_pubkey == recipient()? {
        debug!("fetch_inbox agent");
        debug!("inbox_hash: {}", inbox_hash()?);
        for link in get_links(inbox_hash()?, None)?.into_inner() {
            debug!("fetch_inbox link");
            if let Some(message_entry) = get(link.target, GetOptions::latest())? {
                debug!("fetch_inbox link got");
                let header_address = message_entry.header_address().clone();
                let public_message = PublicMessage::try_from(message_entry)?;
                let message: PerspectiveExpression = public_message.into();
                create_entry(StoredMessage(message))?;
                delete_link(link.create_link_hash)?;
                delete_entry(header_address)?;
            } else {
                error!("Message linked in inbox not retrievable")
            }
        }
        Ok(())
    } else {
        Err(WasmError::Guest(String::from("Only recipient can fetch the inbox")))
    }
}

#[hdk_extern]
fn validate_delete_link(
    validate_delete_link: ValidateDeleteLinkData,
) -> ExternResult<ValidateLinkCallbackResult> {
    let delete_link = validate_delete_link.delete_link;
    let recipient = recipient()?;
    if delete_link.author == recipient {
        Ok(ValidateLinkCallbackResult::Valid)
    } else {
        Ok(ValidateLinkCallbackResult::Invalid("Only recipient is allowed to delete inbox links".to_string()),)
    }
}