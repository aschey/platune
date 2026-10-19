import type { Command } from "commander";
import type { CommandContext } from "../commandContext";
import type { AppCommand } from ".";

export const mount: AppCommand = {
  namespace: "mount",
  name: "get",
  title: "Get registered mount",
  desc: "Shows registered mount",
  action: async (context: CommandContext) => {
    const mount = await context.managementClient.getRegisteredMount();
    console.log(mount);
  },
};
