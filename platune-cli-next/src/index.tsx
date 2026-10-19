import { Command } from "commander";
import { commands } from "./commands";
import "./patchCommander";
import type { CommandContext } from "./commandContext";
import { ManagementClient } from "platune-client";

import { createCliRenderer } from "@opentui/core";
import { createDefaultOpenTuiKeymap } from "@opentui/keymap/opentui";
import { KeymapProvider, useBindings, useKeymap } from "@opentui/keymap/react";

import {
  registerDefaultKeys,
  registerExCommands,
  registerMetadataFields,
} from "@opentui/keymap/addons";
import { createRoot } from "@opentui/react";
import { CommandPalette } from "./components/ui/command-palette";
import { useState } from "react";

const App = () => {
  const commandContext: CommandContext = {
    managementClient: new ManagementClient(),
  };
  useBindings(() => ({
    commands: commands.map((c) => ({
      name: [c.namespace, c.name].filter((c) => !!c).join("."),
      namespace: "excommands",
      aliases: c.aliases,
      title: c.title,
      desc: c.desc,
      run() {
        c.action(commandContext);
      },
    })),
    bindings: [{ key: "q", cmd: "app.quit" }],
  }));

  const keymap = useKeymap();
  const [isOpen, setOpen] = useState(false);

  return (
    <>
      <text onMouseDown={() => setOpen(true)}>show keymap</text>

      <CommandPalette
        isOpen={isOpen}
        onClose={() => setOpen(false)}
        commands={keymap.getCommandEntries().map((c) => ({
          id: c.command.name.substring(1),
          label: c.command.name.substring(1),
          onSelect: () => {
            keymap.dispatchCommand(c.command.name);
          },
        }))}
      />
    </>
  );
};

const runTui = async () => {
  const renderer = await createCliRenderer();
  const keymap = createDefaultOpenTuiKeymap(renderer);
  registerDefaultKeys(keymap);
  registerExCommands(keymap);
  registerMetadataFields(keymap);

  renderer.keyInput.on("keypress", (key) => {
    if (key.name === "`") renderer.console.toggle();
  });
  createRoot(renderer).render(
    <KeymapProvider keymap={keymap}>
      <App />
    </KeymapProvider>,
  );
};

const runCli = () => {
  const commandContext: CommandContext = {
    managementClient: new ManagementClient(),
  };
  const program = new Command();
  program.action(runTui);
  const namespaceMap: Record<string, Command> = {};
  for (let command of commands) {
    let parentCommand = program;
    if (command.namespace) {
      if (!(command.namespace in namespaceMap)) {
        namespaceMap[command.namespace] = program.command(command.namespace);
      }
      parentCommand = namespaceMap[command.namespace]!;
    }
    const newCommand = parentCommand.command(command.name);
    newCommand.action(() => command.action(commandContext));
  }
  program.parse();
};

runCli();
