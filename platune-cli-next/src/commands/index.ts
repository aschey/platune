import type { CommandContext } from "@/commandContext";
import { folders } from "./folders";
import { mount } from "./mount";

export const commands = [folders, mount];

export interface AppCommand {
  namespace?: string;
  name: string;
  action: (context: CommandContext) => void;
  aliases?: string[];
  title: string;
  desc: string;
}
