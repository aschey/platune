import { UnaryCall } from "./grpc_promisify";
import { ManagementClient } from "./management/management_grpc_pb";
import { FoldersMessage } from "./management/management_pb";
import { Empty } from "google-protobuf/google/protobuf/empty_pb.js";
import { credentials } from "@grpc/grpc-js";

async function main() {
  var client = new ManagementClient(
    "unix:///tmp/platune/platuned.sock",
    credentials.createInsecure(),
  );
  var res: FoldersMessage = await UnaryCall(
    client,
    client.getAllFolders,
    new Empty(),
  );

  console.log(res.getFoldersList());
}

main();
