import { Orchestrator, Config, InstallAgentsHapps, TransportConfigType } from "@holochain/tryorama";
import { Perspective } from "@perspect3vism/ad4m"
import path from "path";

const network = {
  transport_pool: [{
    type: TransportConfigType.Quic,
  }],
  bootstrap_service: "https://bootstrap.holo.host"
}
const conductorConfig = Config.gen();
const dm = path.join(__dirname, "../../workdir/direct-message-language.dna");
const installation: InstallAgentsHapps = [
  // agent 0
  [
    // happ 0
    [dm],
  ],
];

const ZOME = "dm"

const orchestrator = new Orchestrator();

orchestrator.registerScenario("create a code", async (s, t) => {
  const [alice] = await s.players([conductorConfig]);
  const [[dm]] = await alice.installAgentsHapps(installation);

  const perspective = new Perspective()

  const proof = {
    signature: "asdfasdfasdf",
    key: "did:test:test#primary"
  }

  const expression = {
    author: "did:test:test",
    timestamp: new Date().toISOString(),
    data: perspective,
    proof
  }

  await dm.cells[0].call(
    ZOME,
    "send",
    expression
  );

  t.ok(true)
  t.equal(true, true)
});

orchestrator.run();
