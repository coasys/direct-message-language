import { DirectMessageAdapter, DID, LanguageContext, MessageCallback, Perspective, PerspectiveExpression, StatusCallback } from "@perspect3vism/ad4m";
import { DNA_NICK } from "./dna";

const DID = "<not templated yet>"
export default class DMAdapter extends DirectMessageAdapter {

  constructor(context: LanguageContext) {
    super()
  }

  recipient(): DID{
    return DID
  }

  async status(timeout: number): Promise<PerspectiveExpression | void> {
    return null
  }

  async sendP2P(message: Perspective): Promise<boolean> {
    return false
  }

  sendInbox(message: Perspective) {

  }

  inbox(): Promise<PerspectiveExpression[]> {

  }

  addMessageCallback(callback: MessageCallback) {

  }

  addStatusCallback(callback: StatusCallback) {

  }
}