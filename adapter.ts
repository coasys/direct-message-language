import { DirectMessageAdapter, DID, HolochainLanguageDelegate, LanguageContext, MessageCallback, Perspective, PerspectiveExpression, StatusCallback } from "@perspect3vism/ad4m";
import { DNA, DNA_NICK } from "./dna";

const DID = "<not templated yet>"

export const sleep = ms => new Promise(r => setTimeout(r, ms))

export default class DMAdapter extends DirectMessageAdapter {
  #context: LanguageContext
  #holochain: HolochainLanguageDelegate;
  #messageCallbacks: MessageCallback[];


  constructor(context: LanguageContext) {
    super()
    this.#context = context
    this.#holochain = context.Holochain as HolochainLanguageDelegate;
    this.#messageCallbacks = []
  }

  async init() {
    const that = this
    await this.#holochain.registerDNAs([{ file: DNA, nick: DNA_NICK }], (signal) => {
      that.#messageCallbacks.forEach(cb => cb(signal))
    });
  }

  recipient(): DID{
    return DID
  }

  async status(): Promise<PerspectiveExpression | void> {
    let status = null
    try {
      status = await this.#holochain.call(DNA_NICK, "direct-message", "get_status")  
    } catch(e) {
      console.debug("DirectMessage Language couldn't get status:", e)
    }
    return status
  }

  async sendP2P(message: Perspective): Promise<boolean> {
    const messageExpression = this.#context.agent.createSignedExpression(message)
    return await this.#holochain.call(DNA_NICK, "direct-message", "send_p2p", messageExpression)
  }

  async sendInbox(message: Perspective) {
    const messageExpression = this.#context.agent.createSignedExpression(message)
    return await this.#holochain.call(DNA_NICK, "direct-message", "send_inbox", messageExpression)
  }

  async inbox(): Promise<PerspectiveExpression[]> {
    await this.#holochain.call(DNA_NICK, "direct-message", "fetch_inbox")
    return await this.#holochain.call(DNA_NICK, "direct-message", "inbox")
  }

  addMessageCallback(callback: MessageCallback) {
    this.#messageCallbacks.push(callback)
  }
}