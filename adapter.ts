import { DirectMessageAdapter, DID, LanguageContext, MessageCallback, Perspective, PerspectiveExpression, StatusCallback } from "@perspect3vism/ad4m";
import { DNA_NICK } from "./dna";

export default class DMAdapter extends DirectMessageAdapter {

  constructor(context: LanguageContext) {
    super()
  }

  recipient(): DID{

  }

  status(timeout: number): Promise<PerspectiveExpression | void> {

  }

  sendP2P(message: Perspective): Promise<boolean> {

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