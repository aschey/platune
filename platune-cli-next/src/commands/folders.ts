import type { Command } from "commander";
import type { CommandContext } from "../commandContext";
import type { AppCommand } from ".";

export const folders: AppCommand = {
  namespace: "folders",
  name: "list",
  title: "List all folders",
  desc: "Shows all configured folders",
  action: async (context: CommandContext) => {
    const folders = await context.managementClient.getAllFolders();
    console.log(folders);
  },
};
