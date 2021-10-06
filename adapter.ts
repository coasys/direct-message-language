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
      console.debug("DM Language got HC signal:", signal)
      let payload = signal.data.payload
      try {
        let string = signal.data.payload.toString()
        let cropped = string.substring(string.indexOf("{"))
        let parsed = JSON.parse(cropped)
        payload = parsed
      } catch(e) {
        console.error(e)
      }
      that.#messageCallbacks.forEach(cb => cb(payload))
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

  async sendP2P(message: Perspective): Promise<PerspectiveExpression|void> {
    try {
      const messageExpression = this.#context.agent.createSignedExpression(message)
      await this.#holochain.call(DNA_NICK, "direct-message", "send_p2p", messageExpression)
      return messageExpression
    } catch(e) {
      console.error("Direct Message Language: Error sending p2p to", recipient_did)
    }
  }

  async sendInbox(message: Perspective): Promise<PerspectiveExpression|void> {
    try {
      const messageExpression = this.#context.agent.createSignedExpression(message)
      await this.#holochain.call(DNA_NICK, "direct-message", "send_inbox", messageExpression)
      return messageExpression
    } catch(e) {
      console.error("Direct Message Language: Error sending to inbox of", recipient_did)
    }
  }

  onlyRecipient() {
    if(recipient_did !== this.#context.agent.did) throw new Error("Only recipient can call this function!")
  }

  async setStatus(status: PerspectiveExpression) {
    this.onlyRecipient()
    const statusExpression = this.#context.agent.createSignedExpression(status)
    await this.#holochain.call(DNA_NICK, "direct-message", "set_status", statusExpression)
  }

  async inbox(filter?: string): Promise<PerspectiveExpression[]> {
    this.onlyRecipient()
    await this.#holochain.call(DNA_NICK, "direct-message", "fetch_inbox", null)
    return await this.#holochain.call(DNA_NICK, "direct-message", "inbox", filter)
  }

  addMessageCallback(callback: MessageCallback) {
    this.onlyRecipient()
    this.#messageCallbacks.push(callback)
  }
}