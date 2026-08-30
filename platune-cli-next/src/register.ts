import type { Command } from "commander";
import type { CommandContext } from "./commandContext";

export function register<C extends Command, T>(
  command: C,
  context: CommandContext,
  plugin: (command: C, context: CommandContext) => T,
): NonNullable<T> extends void ? C : NonNullable<T> {
  const result = plugin(command, context);
  if (result != null) {
    return result as any; // ignore type error due to conditional return
  }
  return command as any; // ignore type error due to conditional return
}

export function defineCommanderPlugin<T>(
  plugin: (command: Command, context: CommandContext) => T,
) {
  return plugin;
}
