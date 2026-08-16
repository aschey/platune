// GENERATED CODE -- DO NOT EDIT!

'use strict';
var grpc = require('@grpc/grpc-js');
var management_pb = require('./management_pb.js');
var google_protobuf_duration_pb = require('google-protobuf/google/protobuf/duration_pb.js');
var google_protobuf_empty_pb = require('google-protobuf/google/protobuf/empty_pb.js');

function serialize_google_protobuf_Empty(arg) {
  if (!(arg instanceof google_protobuf_empty_pb.Empty)) {
    throw new Error('Expected argument of type google.protobuf.Empty');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_google_protobuf_Empty(buffer_arg) {
  return google_protobuf_empty_pb.Empty.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_platune_management_v1_AlbumResponse(arg) {
  if (!(arg instanceof management_pb.AlbumResponse)) {
    throw new Error('Expected argument of type platune.management.v1.AlbumResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_platune_management_v1_AlbumResponse(buffer_arg) {
  return management_pb.AlbumResponse.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_platune_management_v1_FoldersMessage(arg) {
  if (!(arg instanceof management_pb.FoldersMessage)) {
    throw new Error('Expected argument of type platune.management.v1.FoldersMessage');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_platune_management_v1_FoldersMessage(buffer_arg) {
  return management_pb.FoldersMessage.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_platune_management_v1_GetDeletedResponse(arg) {
  if (!(arg instanceof management_pb.GetDeletedResponse)) {
    throw new Error('Expected argument of type platune.management.v1.GetDeletedResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_platune_management_v1_GetDeletedResponse(buffer_arg) {
  return management_pb.GetDeletedResponse.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_platune_management_v1_IdMessage(arg) {
  if (!(arg instanceof management_pb.IdMessage)) {
    throw new Error('Expected argument of type platune.management.v1.IdMessage');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_platune_management_v1_IdMessage(buffer_arg) {
  return management_pb.IdMessage.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_platune_management_v1_LookupRequest(arg) {
  if (!(arg instanceof management_pb.LookupRequest)) {
    throw new Error('Expected argument of type platune.management.v1.LookupRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_platune_management_v1_LookupRequest(buffer_arg) {
  return management_pb.LookupRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_platune_management_v1_LookupResponse(arg) {
  if (!(arg instanceof management_pb.LookupResponse)) {
    throw new Error('Expected argument of type platune.management.v1.LookupResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_platune_management_v1_LookupResponse(buffer_arg) {
  return management_pb.LookupResponse.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_platune_management_v1_PathMessage(arg) {
  if (!(arg instanceof management_pb.PathMessage)) {
    throw new Error('Expected argument of type platune.management.v1.PathMessage');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_platune_management_v1_PathMessage(buffer_arg) {
  return management_pb.PathMessage.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_platune_management_v1_Progress(arg) {
  if (!(arg instanceof management_pb.Progress)) {
    throw new Error('Expected argument of type platune.management.v1.Progress');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_platune_management_v1_Progress(buffer_arg) {
  return management_pb.Progress.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_platune_management_v1_RegisteredMountMessage(arg) {
  if (!(arg instanceof management_pb.RegisteredMountMessage)) {
    throw new Error('Expected argument of type platune.management.v1.RegisteredMountMessage');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_platune_management_v1_RegisteredMountMessage(buffer_arg) {
  return management_pb.RegisteredMountMessage.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_platune_management_v1_SearchRequest(arg) {
  if (!(arg instanceof management_pb.SearchRequest)) {
    throw new Error('Expected argument of type platune.management.v1.SearchRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_platune_management_v1_SearchRequest(buffer_arg) {
  return management_pb.SearchRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_platune_management_v1_SearchResponse(arg) {
  if (!(arg instanceof management_pb.SearchResponse)) {
    throw new Error('Expected argument of type platune.management.v1.SearchResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_platune_management_v1_SearchResponse(buffer_arg) {
  return management_pb.SearchResponse.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_platune_management_v1_SongResponse(arg) {
  if (!(arg instanceof management_pb.SongResponse)) {
    throw new Error('Expected argument of type platune.management.v1.SongResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_platune_management_v1_SongResponse(buffer_arg) {
  return management_pb.SongResponse.deserializeBinary(new Uint8Array(buffer_arg));
}


var ManagementService = exports.ManagementService = {
  startSync: {
    path: '/platune.management.v1.Management/StartSync',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  addFolders: {
    path: '/platune.management.v1.Management/AddFolders',
    requestStream: false,
    responseStream: false,
    requestType: management_pb.FoldersMessage,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_platune_management_v1_FoldersMessage,
    requestDeserialize: deserialize_platune_management_v1_FoldersMessage,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  getAllFolders: {
    path: '/platune.management.v1.Management/GetAllFolders',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: management_pb.FoldersMessage,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_platune_management_v1_FoldersMessage,
    responseDeserialize: deserialize_platune_management_v1_FoldersMessage,
  },
  registerMount: {
    path: '/platune.management.v1.Management/RegisterMount',
    requestStream: false,
    responseStream: false,
    requestType: management_pb.RegisteredMountMessage,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_platune_management_v1_RegisteredMountMessage,
    requestDeserialize: deserialize_platune_management_v1_RegisteredMountMessage,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  getRegisteredMount: {
    path: '/platune.management.v1.Management/GetRegisteredMount',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: management_pb.RegisteredMountMessage,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_platune_management_v1_RegisteredMountMessage,
    responseDeserialize: deserialize_platune_management_v1_RegisteredMountMessage,
  },
  search: {
    path: '/platune.management.v1.Management/Search',
    requestStream: true,
    responseStream: true,
    requestType: management_pb.SearchRequest,
    responseType: management_pb.SearchResponse,
    requestSerialize: serialize_platune_management_v1_SearchRequest,
    requestDeserialize: deserialize_platune_management_v1_SearchRequest,
    responseSerialize: serialize_platune_management_v1_SearchResponse,
    responseDeserialize: deserialize_platune_management_v1_SearchResponse,
  },
  lookup: {
    path: '/platune.management.v1.Management/Lookup',
    requestStream: false,
    responseStream: false,
    requestType: management_pb.LookupRequest,
    responseType: management_pb.LookupResponse,
    requestSerialize: serialize_platune_management_v1_LookupRequest,
    requestDeserialize: deserialize_platune_management_v1_LookupRequest,
    responseSerialize: serialize_platune_management_v1_LookupResponse,
    responseDeserialize: deserialize_platune_management_v1_LookupResponse,
  },
  getSongByPath: {
    path: '/platune.management.v1.Management/GetSongByPath',
    requestStream: false,
    responseStream: false,
    requestType: management_pb.PathMessage,
    responseType: management_pb.SongResponse,
    requestSerialize: serialize_platune_management_v1_PathMessage,
    requestDeserialize: deserialize_platune_management_v1_PathMessage,
    responseSerialize: serialize_platune_management_v1_SongResponse,
    responseDeserialize: deserialize_platune_management_v1_SongResponse,
  },
  getAlbumsByAlbumArtists: {
    path: '/platune.management.v1.Management/GetAlbumsByAlbumArtists',
    requestStream: false,
    responseStream: false,
    requestType: management_pb.IdMessage,
    responseType: management_pb.AlbumResponse,
    requestSerialize: serialize_platune_management_v1_IdMessage,
    requestDeserialize: deserialize_platune_management_v1_IdMessage,
    responseSerialize: serialize_platune_management_v1_AlbumResponse,
    responseDeserialize: deserialize_platune_management_v1_AlbumResponse,
  },
  getDeleted: {
    path: '/platune.management.v1.Management/GetDeleted',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: management_pb.GetDeletedResponse,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_platune_management_v1_GetDeletedResponse,
    responseDeserialize: deserialize_platune_management_v1_GetDeletedResponse,
  },
  deleteTracks: {
    path: '/platune.management.v1.Management/DeleteTracks',
    requestStream: false,
    responseStream: false,
    requestType: management_pb.IdMessage,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_platune_management_v1_IdMessage,
    requestDeserialize: deserialize_platune_management_v1_IdMessage,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  subscribeEvents: {
    path: '/platune.management.v1.Management/SubscribeEvents',
    requestStream: false,
    responseStream: true,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: management_pb.Progress,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_platune_management_v1_Progress,
    responseDeserialize: deserialize_platune_management_v1_Progress,
  },
};

exports.ManagementClient = grpc.makeGenericClientConstructor(ManagementService, 'Management');
