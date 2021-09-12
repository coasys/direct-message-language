import type { Address, Agent, Language, HolochainLanguageDelegate, LanguageContext, Interaction, ExpressionUI } from "@perspect3vism/ad4m";
import { DNA, DNA_NICK } from "./dna";
import DMAdapter from "./adapter";

export const name = "direct-message-language";

function interactions(expression: Address): Interaction[] {
  return [];
}

export default async function create(context: LanguageContext): Promise<Language> {
  const Holochain = context.Holochain as HolochainLanguageDelegate;
  await Holochain.registerDNAs([{ file: DNA, nick: DNA_NICK }]);

  const directMessageAdapter = new DMAdapter(context);

  return {
    name,
    directMessageAdapter,
    interactions,
  } as Language;
}
