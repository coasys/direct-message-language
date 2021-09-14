mod ad4m;
mod test_inspect;

use hdk::prelude::*;
use ad4m::PerspectiveExpression;
use test_inspect::Recipient;
use test_inspect::get_test_recipient;

#[hdk_entry(id = "message_wrapper", visibility = "public")]
#[derive(Clone)]
enum MessageWrapper {
    StatusRequest,
    StatusResponse(PerspectiveExpression),
    DirectMessage(PerspectiveExpression)
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
    Recipient::entry_def()
];

#[hdk_extern]
fn init(_: ()) -> ExternResult<InitCallbackResult> {
    debug!("INIT called");
    let mut functions: GrantedFunctions = BTreeSet::new();
    functions.insert((zome_info()?.zome_name, "recv_remote_signal".into()));

    //Create open cap grant to allow agents to send signals of links to each other
    create_cap_grant(CapGrantEntry {
        tag: "".into(),
        // empty access converts to unrestricted
        access: ().into(),
        functions,
    })?;
    Ok(InitCallbackResult::Pass)
}

#[hdk_extern]
fn recv_remote_signal(signal: SerializedBytes) -> ExternResult<()> {
    debug!("RECEIVEING MESSAGE...");
    //debug!("{}", String::from(signal));
    emit_signal(String::from("recv_remote_signal"))?;
    match MessageWrapper::try_from(signal)? {
        MessageWrapper::StatusRequest => Err(WasmError::Guest(String::from("StatusRequest not implemented"))),
        MessageWrapper::StatusResponse(_message) => Err(WasmError::Guest(String::from("StatusResponse not implemented"))),
        MessageWrapper::DirectMessage(message) => {
            let json = serde_json::to_string(&message).unwrap();
            Ok(emit_signal(&SerializedBytes::try_from(Signal{json})?)?)
        }
    }
}

fn recipient() -> ExternResult<AgentPubKey> {
    if let Some(recipient) = get_test_recipient(())? {
        Ok(recipient.get())
    } else {
        let properties = Properties::try_from(zome_info()?.properties)?;
        Ok(properties.recipient_hc_agent_pubkey)
    }
}

#[hdk_extern]
pub fn send(message: PerspectiveExpression) -> ExternResult<()> {
    debug!("SENDING MESSAGE...");
    remote_signal(SerializedBytes::try_from(MessageWrapper::DirectMessage(message))?, vec![recipient()?])
}

