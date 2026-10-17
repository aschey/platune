import type { Command } from "commander";
import type { CommandContext } from "./commandContext";

export function register<C extends Command, T>(
  command: C,
  context: CommandContext,
  plugin: (command: C, context: CommandContext) => T,
): T {
  return plugin(command, context);
}

export function defineCommanderPlugin<T>(
  plugin: (command: Command, context: CommandContext) => T,
) {
  return plugin;
}
