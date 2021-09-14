import { Orchestrator, Config, InstallAgentsHapps, TransportConfigType } from "@holochain/tryorama";
import { Perspective } from "@perspect3vism/ad4m"
import path from "path";

export const sleep = ms => new Promise(r => setTimeout(r, ms))

const conductorConfig = Config.gen();
const dm = path.join(__dirname, "../../workdir/direct-message-language.dna");
const installation: InstallAgentsHapps = [
  // agent 0
  [
    // happ 0
    [dm],
  ],
];

const ZOME = "direct-message"

const orchestrator = new Orchestrator();

orchestrator.registerScenario("send direct message", async (s, t) => {
  const [alice, bob] = await s.players([conductorConfig, conductorConfig]);

  let alice_last_signal
  alice.setSignalHandler(signal => {
    let payload = signal.data.payload
    try {
      let cropped = signal.data.payload.toString().substring(7)
      //console.log("CROPPED:", cropped)
      let parsed = JSON.parse(cropped)
      //console.log("PARSED:", parsed)
      payload = parsed
    } catch(e) {
      //console.error(e)
    }
    console.log("SIGNAL @ALICE:", payload)
    alice_last_signal = payload
  })

  bob.setSignalHandler(signal => {
    console.log("SIGNAL @BOB:", signal.data.payload.toString())
  })
  const [[alice_dm]] = await alice.installAgentsHapps(installation);
  const [[bob_dm]] = await bob.installAgentsHapps(installation);

  await s.shareAllNodes([alice, bob])

  const alice_agent_pubkey = alice_dm.agent
  await alice_dm.cells[0].call(ZOME, "set_test_recipient", alice_agent_pubkey);
  await bob_dm.cells[0].call(ZOME, "set_test_recipient", alice_agent_pubkey);
  const stored_recipient = await bob_dm.cells[0].call(ZOME, "get_test_recipient");
  t.equal(stored_recipient.toString(), alice_agent_pubkey.toString())

  const expression = {
    author: "did:test:test",
    timestamp: new Date().toISOString(),
    data: {
      links: []
    },
    proof: {
      signature: "asdfasdfasdf",
      key: "did:test:test#primary"
    }
  }

  await bob_dm.cells[0].call(
    ZOME,
    "send",
    expression
  );

  await sleep(1000)

  console.log(alice_last_signal)
  console.log(expression)
  t.deepEqual(alice_last_signal, expression)
});

orchestrator.run();