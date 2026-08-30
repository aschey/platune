import { Command } from "commander";
import { commands } from "./commands";
import "./patchCommander";
import type { CommandContext } from "./commandContext";
import { ManagementClient } from "platune-client";

const runCli = () => {
  const commandContext: CommandContext = {
    managementClient: new ManagementClient(),
  };
  const program = new Command();
  for (let command of commands) {
    program.register(commandContext, command);
  }
  program.parse();
};

runCli();

// import { createCliRenderer } from "@opentui/core";
// import { createDefaultOpenTuiKeymap } from "@opentui/keymap/opentui";
// import {
//   KeymapProvider,
//   useBindings,
//   useKeymap,
//   useKeymapSelector,
// } from "@opentui/keymap/solid";
// import { render } from "@opentui/solid";
// import {
//   registerDefaultKeys,
//   registerExCommands,
//   registerMetadataFields,
// } from "@opentui/keymap/addons";
// import type { CommandContext } from "./commandContext";
// import { ManagementClient } from "platune-client";

// const renderer = await createCliRenderer();
// const keymap = createDefaultOpenTuiKeymap(renderer);
// registerDefaultKeys(keymap);
// registerExCommands(keymap);
// registerMetadataFields(keymap);

// function App() {
//   const k = useKeymap();
//   useBindings(() => ({
//     commands: [
//       {
//         name: "app.quit",
//         run() {
//           renderer.destroy();
//         },
//       },
//       {
//         name: "app.dispatch",
//         run() {
//           let res = k.dispatchCommand(":w");

//           if (!res || !res.ok) {
//             throw Error(res.reason);
//           }
//         },
//       },
//       {
//         name: "write",
//         namespace: "excommands",
//         aliases: ["w"],
//         title: "Write file",
//         desc: "Write file",
//         run() {
//           renderer.destroy();
//         },
//       },
//     ],
//     bindings: [
//       { key: "q", cmd: "app.quit" },
//       { key: "w", cmd: "app.dispatch" },
//     ],
//   }));

//   const entries = useKeymapSelector((k) => k.getCommandEntries());

//   return <text>{entries()[0]?.command.name}</text>;
//   // return entries()
//   //   .flatMap((k) => k.bindings)
//   //   .flatMap((k) => k.sequence)
//   //   .map((k) => <text>a {k.display}</text>);
// }

// await render(
//   () => (
//     <KeymapProvider keymap={keymap}>
//       <App />
//     </KeymapProvider>
//   ),
//   renderer,
// );
