import type { CommandContext } from "@/commandContext";
import { folders } from "./folders";

export const commands = [folders];

export interface AppCommand {
  namespace?: string;
  name: string;
  action: (context: CommandContext) => void;
  aliases?: string[];
  title: string;
  desc: string;
}
