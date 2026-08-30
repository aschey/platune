import type { Command } from "commander";
import { render } from "@opentui/solid";
import { KeymapProvider, useBindings } from "@opentui/keymap/solid";
import type { CommandContext } from "../commandContext";

export const folders = (command: Command, context: CommandContext) => {
  let folders = command.command("folders");
  folders.command("list").action(async () => {
    const folders = await context.managementClient.getAllFolders();
    console.log(folders);
  });
};
