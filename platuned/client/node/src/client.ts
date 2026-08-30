import { UnaryCall } from "./grpc_promisify";
import { ManagementClient as ManagementRpc } from "./management/management_grpc_pb";
import { FoldersMessage } from "./management/management_pb";
import { Empty } from "google-protobuf/google/protobuf/empty_pb.js";
import { credentials } from "@grpc/grpc-js";

async function main() {
  var client = new ManagementRpc(
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

// main();

export class ManagementClient {
  private client: ManagementRpc;

  constructor() {
    let client = new ManagementRpc(
      "unix:///tmp/platune/platuned.sock",
      credentials.createInsecure(),
    );
    this.client = client;
  }

  getAllFolders = async () => {
    var res: FoldersMessage = await UnaryCall(
      this.client,
      this.client.getAllFolders,
      new Empty(),
    );
    return res.getFoldersList();
  };
}
