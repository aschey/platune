import { Command } from "commander";
import { register } from "./register.ts";
import type { CommandContext } from "./commandContext.ts";

declare module "commander" {
  interface Command {
    register<T>(
      context: CommandContext,
      plugin: (command: this, context: CommandContext) => T,
    ): NonNullable<T> extends void ? this : NonNullable<T>;
  }
}

Command.prototype.register = function <T>(
  this: Command,
  context: CommandContext,
  plugin: (command: typeof this, context: CommandContext) => T,
) {
  return register(this, context, plugin);
};
