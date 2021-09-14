use hdk::prelude::*;

#[hdk_entry(
    id = "recipient",
    visibility = "private"
)]
pub struct Recipient(AgentPubKey);

impl Recipient {
    pub fn get(&self) -> AgentPubKey {
        return self.0.clone()
    }
}

#[hdk_extern]
pub fn set_test_recipient(recipient: AgentPubKey) -> ExternResult<()> {
    create_entry(Recipient(recipient))?;
    Ok(())
}

#[hdk_extern]
pub fn get_test_recipient(_: ()) -> ExternResult<Option<Recipient>> {
    let mut filter = QueryFilter::new();
    filter.entry_type = Some(entry_type!(Recipient)?);
    filter.include_entries = true;
    if let Some(element) = query(filter)?.pop() {
        let recipient = Recipient::try_from(element)?;
                return Ok(Some(recipient));
    }
    return Ok(None)
}