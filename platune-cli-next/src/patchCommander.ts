import { Command } from "commander";
import { register } from "./register.ts";
import type { CommandContext } from "./commandContext.ts";
import type { AppCommand } from "./commands/index.ts";

declare module "commander" {
  interface Command {
    register(
      context: CommandContext,
      plugin: (command: this, context: CommandContext) => AppCommand,
    ): AppCommand;
  }
}

Command.prototype.register = function (
  this: Command,
  context: CommandContext,
  plugin: (command: typeof this, context: CommandContext) => AppCommand,
) {
  return register(this, context, plugin);
};
