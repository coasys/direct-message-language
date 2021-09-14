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
pub fn send(message: PerspectiveExpression) -> ExternResult<()> {
    debug!("SENDING MESSAGE...");
    remote_signal(SerializedBytes::try_from(message)?, vec![recipient()?])
}

