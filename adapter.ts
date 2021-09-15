import { DirectMessageAdapter, HolochainLanguageDelegate, LanguageContext, MessageCallback, Perspective, PerspectiveExpression, StatusCallback } from "@perspect3vism/ad4m";
import { DNA, DNA_NICK } from "./dna";

const recipient_did = "<not templated yet>"

export const sleep = ms => new Promise(r => setTimeout(r, ms))

export default class DMAdapter implements DirectMessageAdapter {
  #context: LanguageContext
  #holochain: HolochainLanguageDelegate;
  #messageCallbacks: MessageCallback[];


  constructor(context: LanguageContext) {
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

  recipient(): string{
    return recipient_did
  }

  async status(): Promise<PerspectiveExpression | void> {
    let status = null
    try {
      status = await this.#holochain.call(DNA_NICK, "direct-message", "get_status", null)  
    } catch(e) {
      console.debug("DirectMessage Language couldn't get status:", e)
    }
    return status
  }

  async sendP2P(message: Perspective): Promise<boolean> {
    const messageExpression = this.#context.agent.createSignedExpression(message)
    await this.#holochain.call(DNA_NICK, "direct-message", "send_p2p", messageExpression)
    return true
  }

  async sendInbox(message: Perspective) {
    const messageExpression = this.#context.agent.createSignedExpression(message)
    return await this.#holochain.call(DNA_NICK, "direct-message", "send_inbox", messageExpression)
  }

  onlyRecipient() {
    if(recipient_did !== this.#context.agent.did) throw new Error("Only recipient can call this function!")
  }

  async setStatus(status: PerspectiveExpression) {
    this.onlyRecipient()
    const statusExpression = this.#context.agent.createSignedExpression(status)
    await this.#holochain.call(DNA_NICK, "direct-message", "set_status", statusExpression)
  }

  async inbox(): Promise<PerspectiveExpression[]> {
    this.onlyRecipient()
    await this.#holochain.call(DNA_NICK, "direct-message", "fetch_inbox", null)
    return await this.#holochain.call(DNA_NICK, "direct-message", "inbox", null)
  }

  addMessageCallback(callback: MessageCallback) {
    this.onlyRecipient()
    this.#messageCallbacks.push(callback)
  }
}