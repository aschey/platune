/**
 * Promise-based wrapper around @grpc/grpc-js API.
 * from https://gist.githubusercontent.com/paskozdilar/196b212a1df66463487fa2b75dde049c/raw/ba7b113080d7ad180ad1b5dcad7b0400bbb6e445/promisify.ts
 *
 *
 * USAGE:
 *
 * let client: grpc.Client;
 *
 * // unary
 * let response: ResponseType = await UnaryCall(client, client.unaryMethod, request)
 *
 * // client stream
 * let response = await ClientStream(client, client.clientStreamMethod, async function*() {
 *    yield request1;
 *    yield request2;
 * });
 *
 * // server stream
 * for await (let response of ServerStream(client, client.serverStreamMethod, request)) {
 *    // ...
 * }
 *
 * // bidirectional stream
 * for await (let response of Bidirectional(client, client.bidirectionalMethod, async function*() {
 *    yield request1;
 *    yield request2;
 * }) {
 *    // ...
 * }
 *
 *
 * NOTE:
 *
 * Due to TypeScript's type inference limitations, return values must be
 * explicitly annotated for type inference/checking to work.
 */

import * as grpc from "@grpc/grpc-js";

type UnaryCall<ClientType, RequestType, ResponseType> = (
  this: ClientType,
  request: RequestType,
  callback: (error: grpc.ServiceError | null, response: ResponseType) => void,
) => grpc.ClientUnaryCall;

type ClientStream<ClientType, RequestType, ResponseType> = (
  this: ClientType,
  callback: (error: grpc.ServiceError | null, response: ResponseType) => void,
) => grpc.ClientWritableStream<RequestType>;

type ServerStream<ClientType, RequestType, ResponseType> = (
  this: ClientType,
  request: RequestType,
  options?: Partial<grpc.CallOptions>,
) => grpc.ClientReadableStream<ResponseType>;

type Bidirectional<ClientType, RequestType, ResponseType> = (
  this: ClientType,
) => grpc.ClientDuplexStream<RequestType, ResponseType>;

export async function UnaryCall<
  ClientType extends grpc.Client,
  RequestType,
  ResponseType,
>(
  client: ClientType,
  method: UnaryCall<ClientType, RequestType, ResponseType>,
  request: RequestType,
  signal?: AbortSignal,
): Promise<ResponseType> {
  return await new Promise<ResponseType>((resolve, reject) => {
    const call = method.call(client, request, (error, response) => {
      if (error !== null) {
        reject(error);
      } else {
        resolve(response);
      }
    });
    if (signal !== undefined) {
      signal.addEventListener("abort", () => call.cancel());
    }
  });
}

export async function ClientStream<
  ClientType extends grpc.Client,
  RequestType,
  ResponseType,
>(
  client: ClientType,
  method: ClientStream<ClientType, RequestType, ResponseType>,
  requests: () => AsyncGenerator<RequestType, void, unknown>,
  signal?: AbortSignal,
): Promise<ResponseType> {
  return await new Promise<ResponseType>(async (resolve, reject) => {
    const stream = method.call(client, (error, response) => {
      if (error !== null) {
        reject(error);
      } else {
        resolve(response);
      }
    });
    if (signal !== undefined) {
      signal.addEventListener("abort", () => stream.cancel());
    }
    try {
      for await (const request of requests()) {
        await new Promise<void>((resolve, reject) => {
          if (
            !stream.write(request, (error?: Error) => {
              if (error !== undefined) {
                reject(error);
              }
            })
          ) {
            stream.once("drain", resolve);
          } else {
            resolve();
          }
        });
      }
      stream.end();
    } catch (error) {
      reject(error);
    }
  });
}

export async function* ServerStream<ClientType, RequestType, ResponseType>(
  client: ClientType,
  method: ServerStream<ClientType, RequestType, ResponseType>,
  request: RequestType,
  signal?: AbortSignal,
): AsyncGenerator<ResponseType> {
  const stream = method.call(client, request);

  if (signal !== undefined) {
    signal.addEventListener("abort", () => stream.cancel());
  }

  let resolve: (response?: ResponseType) => void;
  let reject: (error: Error) => void;
  let promise = new Promise<ResponseType | undefined>((_resolve, _reject) => {
    resolve = _resolve;
    reject = _reject;
  });

  let buffer: ResponseType[] = [];
  let ready = true;

  stream.on("data", (response) => {
    // if not ready, push to buffer
    if (!ready) {
      buffer.push(response);
      return;
    }
    ready = false;

    // otherwise
    resolve(response);
    promise = new Promise<ResponseType | undefined>((_resolve, _reject) => {
      resolve = _resolve;
      reject = _reject;
    });
  });
  stream.on("end", () => resolve());
  stream.on("error", (error) => reject(error));

  while (true) {
    // drain buffer
    while (buffer.length > 0) {
      yield buffer.shift()!;
    }

    // wait for data
    ready = true;
    const response = await promise;
    if (response === undefined) {
      break;
    }
    yield response;
  }
}

export async function* Bidirectional<ClientType, RequestType, ResponseType>(
  client: ClientType,
  method: Bidirectional<ClientType, RequestType, ResponseType>,
  requests: () => AsyncGenerator<RequestType, void, unknown>,
  signal?: AbortSignal,
): AsyncGenerator<ResponseType> {
  const stream = method.call(client);

  if (signal !== undefined) {
    signal.addEventListener("abort", () => stream.cancel());
  }

  const sendDataInBackground = async () => {
    try {
      for await (const request of requests()) {
        await new Promise<void>((resolve, reject) => {
          const draining = !stream.write(request, (error?: Error) => {
            if (error !== undefined) {
              reject(error);
            }
          });
          if (draining) {
            stream.once("drain", resolve);
          } else {
            resolve();
          }
        });
      }
    } catch {
    } finally {
      stream.end();
    }
  };
  sendDataInBackground();

  let resolve: (response?: ResponseType) => void;
  let reject: (error: Error) => void;
  let promise = new Promise<ResponseType | undefined>((_resolve, _reject) => {
    resolve = _resolve;
    reject = _reject;
  });

  let buffer: ResponseType[] = [];
  let ready = true;

  stream.on("data", (response) => {
    // if not ready, push to buffer
    if (!ready) {
      buffer.push(response);
      return;
    }
    ready = false;

    // otherwise
    resolve(response);
    promise = new Promise<ResponseType | undefined>((_resolve, _reject) => {
      resolve = _resolve;
      reject = _reject;
    });
  });
  stream.on("end", () => resolve());
  stream.on("error", (error) => reject(error));

  while (true) {
    // drain buffer
    while (buffer.length > 0) {
      yield buffer.shift()!;
    }

    // wait for data
    ready = true;
    const response = await promise;
    if (response === undefined) {
      break;
    }
    yield response;
  }
}
