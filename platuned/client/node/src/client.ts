import { UnaryCall } from "./grpc_promisify";
import { platune } from "./management/management";
import { google } from "./management/google/protobuf/empty";
import { credentials } from "@grpc/grpc-js";

async function main() {
  var client = new platune.management.v1.ManagementClient(
    "unix:///tmp/platune/platuned.sock",
    credentials.createInsecure(),
  );
  var res: platune.management.v1.FoldersMessage | undefined = await UnaryCall(
    client,
    client.GetAllFolders,
    new google.protobuf.Empty(),
  );
  if (res) {
    console.log(res);
  }
}

// main();

export class ManagementClient {
  private client: platune.management.v1.ManagementClient;

  constructor() {
    let client = new platune.management.v1.ManagementClient(
      "unix:///tmp/platune/platuned.sock",
      credentials.createInsecure(),
    );
    this.client = client;
  }

  getAllFolders = async () => {
    let res: platune.management.v1.FoldersMessage | undefined = await UnaryCall(
      this.client,
      this.client.GetAllFolders,
      new google.protobuf.Empty(),
    );
    return res?.folders;
  };

  getRegisteredMount = async () => {
    let res: platune.management.v1.RegisteredMountMessage | undefined =
      await UnaryCall(
        this.client,
        this.client.GetRegisteredMount,
        new google.protobuf.Empty(),
      );
    return res?.mount;
  };
}
