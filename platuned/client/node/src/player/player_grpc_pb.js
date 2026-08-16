// GENERATED CODE -- DO NOT EDIT!

'use strict';
var grpc = require('@grpc/grpc-js');
var player_pb = require('./player_pb.js');
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

function serialize_platune_player_v1_DevicesResponse(arg) {
  if (!(arg instanceof player_pb.DevicesResponse)) {
    throw new Error('Expected argument of type platune.player.v1.DevicesResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_platune_player_v1_DevicesResponse(buffer_arg) {
  return player_pb.DevicesResponse.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_platune_player_v1_EventResponse(arg) {
  if (!(arg instanceof player_pb.EventResponse)) {
    throw new Error('Expected argument of type platune.player.v1.EventResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_platune_player_v1_EventResponse(buffer_arg) {
  return player_pb.EventResponse.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_platune_player_v1_QueueRequest(arg) {
  if (!(arg instanceof player_pb.QueueRequest)) {
    throw new Error('Expected argument of type platune.player.v1.QueueRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_platune_player_v1_QueueRequest(buffer_arg) {
  return player_pb.QueueRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_platune_player_v1_SeekRequest(arg) {
  if (!(arg instanceof player_pb.SeekRequest)) {
    throw new Error('Expected argument of type platune.player.v1.SeekRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_platune_player_v1_SeekRequest(buffer_arg) {
  return player_pb.SeekRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_platune_player_v1_SetOutputDeviceRequest(arg) {
  if (!(arg instanceof player_pb.SetOutputDeviceRequest)) {
    throw new Error('Expected argument of type platune.player.v1.SetOutputDeviceRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_platune_player_v1_SetOutputDeviceRequest(buffer_arg) {
  return player_pb.SetOutputDeviceRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_platune_player_v1_SetVolumeRequest(arg) {
  if (!(arg instanceof player_pb.SetVolumeRequest)) {
    throw new Error('Expected argument of type platune.player.v1.SetVolumeRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_platune_player_v1_SetVolumeRequest(buffer_arg) {
  return player_pb.SetVolumeRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_platune_player_v1_StatusResponse(arg) {
  if (!(arg instanceof player_pb.StatusResponse)) {
    throw new Error('Expected argument of type platune.player.v1.StatusResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_platune_player_v1_StatusResponse(buffer_arg) {
  return player_pb.StatusResponse.deserializeBinary(new Uint8Array(buffer_arg));
}


var PlayerService = exports.PlayerService = {
  setQueue: {
    path: '/platune.player.v1.Player/SetQueue',
    requestStream: false,
    responseStream: false,
    requestType: player_pb.QueueRequest,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_platune_player_v1_QueueRequest,
    requestDeserialize: deserialize_platune_player_v1_QueueRequest,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  addToQueue: {
    path: '/platune.player.v1.Player/AddToQueue',
    requestStream: false,
    responseStream: false,
    requestType: player_pb.QueueRequest,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_platune_player_v1_QueueRequest,
    requestDeserialize: deserialize_platune_player_v1_QueueRequest,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  pause: {
    path: '/platune.player.v1.Player/Pause',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  stop: {
    path: '/platune.player.v1.Player/Stop',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  resume: {
    path: '/platune.player.v1.Player/Resume',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  toggle: {
    path: '/platune.player.v1.Player/Toggle',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  seek: {
    path: '/platune.player.v1.Player/Seek',
    requestStream: false,
    responseStream: false,
    requestType: player_pb.SeekRequest,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_platune_player_v1_SeekRequest,
    requestDeserialize: deserialize_platune_player_v1_SeekRequest,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  setVolume: {
    path: '/platune.player.v1.Player/SetVolume',
    requestStream: false,
    responseStream: false,
    requestType: player_pb.SetVolumeRequest,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_platune_player_v1_SetVolumeRequest,
    requestDeserialize: deserialize_platune_player_v1_SetVolumeRequest,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  next: {
    path: '/platune.player.v1.Player/Next',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  previous: {
    path: '/platune.player.v1.Player/Previous',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  getCurrentStatus: {
    path: '/platune.player.v1.Player/GetCurrentStatus',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: player_pb.StatusResponse,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_platune_player_v1_StatusResponse,
    responseDeserialize: deserialize_platune_player_v1_StatusResponse,
  },
  subscribeEvents: {
    path: '/platune.player.v1.Player/SubscribeEvents',
    requestStream: false,
    responseStream: true,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: player_pb.EventResponse,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_platune_player_v1_EventResponse,
    responseDeserialize: deserialize_platune_player_v1_EventResponse,
  },
  listOutputDevices: {
    path: '/platune.player.v1.Player/ListOutputDevices',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: player_pb.DevicesResponse,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_platune_player_v1_DevicesResponse,
    responseDeserialize: deserialize_platune_player_v1_DevicesResponse,
  },
  setOutputDevice: {
    path: '/platune.player.v1.Player/SetOutputDevice',
    requestStream: false,
    responseStream: false,
    requestType: player_pb.SetOutputDeviceRequest,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_platune_player_v1_SetOutputDeviceRequest,
    requestDeserialize: deserialize_platune_player_v1_SetOutputDeviceRequest,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
};

exports.PlayerClient = grpc.makeGenericClientConstructor(PlayerService, 'Player');
