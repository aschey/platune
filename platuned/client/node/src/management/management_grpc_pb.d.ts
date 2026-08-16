// package: platune.management.v1
// file: management.proto

/* tslint:disable */
/* eslint-disable */

import * as grpc from "@grpc/grpc-js";
import * as management_pb from "./management_pb";
import * as google_protobuf_duration_pb from "google-protobuf/google/protobuf/duration_pb";
import * as google_protobuf_empty_pb from "google-protobuf/google/protobuf/empty_pb";

interface IManagementService extends grpc.ServiceDefinition<grpc.UntypedServiceImplementation> {
    startSync: IManagementService_IStartSync;
    addFolders: IManagementService_IAddFolders;
    getAllFolders: IManagementService_IGetAllFolders;
    registerMount: IManagementService_IRegisterMount;
    getRegisteredMount: IManagementService_IGetRegisteredMount;
    search: IManagementService_ISearch;
    lookup: IManagementService_ILookup;
    getSongByPath: IManagementService_IGetSongByPath;
    getAlbumsByAlbumArtists: IManagementService_IGetAlbumsByAlbumArtists;
    getDeleted: IManagementService_IGetDeleted;
    deleteTracks: IManagementService_IDeleteTracks;
    subscribeEvents: IManagementService_ISubscribeEvents;
}

interface IManagementService_IStartSync extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty> {
    path: "/platune.management.v1.Management/StartSync";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementService_IAddFolders extends grpc.MethodDefinition<management_pb.FoldersMessage, google_protobuf_empty_pb.Empty> {
    path: "/platune.management.v1.Management/AddFolders";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<management_pb.FoldersMessage>;
    requestDeserialize: grpc.deserialize<management_pb.FoldersMessage>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementService_IGetAllFolders extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, management_pb.FoldersMessage> {
    path: "/platune.management.v1.Management/GetAllFolders";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<management_pb.FoldersMessage>;
    responseDeserialize: grpc.deserialize<management_pb.FoldersMessage>;
}
interface IManagementService_IRegisterMount extends grpc.MethodDefinition<management_pb.RegisteredMountMessage, google_protobuf_empty_pb.Empty> {
    path: "/platune.management.v1.Management/RegisterMount";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<management_pb.RegisteredMountMessage>;
    requestDeserialize: grpc.deserialize<management_pb.RegisteredMountMessage>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementService_IGetRegisteredMount extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, management_pb.RegisteredMountMessage> {
    path: "/platune.management.v1.Management/GetRegisteredMount";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<management_pb.RegisteredMountMessage>;
    responseDeserialize: grpc.deserialize<management_pb.RegisteredMountMessage>;
}
interface IManagementService_ISearch extends grpc.MethodDefinition<management_pb.SearchRequest, management_pb.SearchResponse> {
    path: "/platune.management.v1.Management/Search";
    requestStream: true;
    responseStream: true;
    requestSerialize: grpc.serialize<management_pb.SearchRequest>;
    requestDeserialize: grpc.deserialize<management_pb.SearchRequest>;
    responseSerialize: grpc.serialize<management_pb.SearchResponse>;
    responseDeserialize: grpc.deserialize<management_pb.SearchResponse>;
}
interface IManagementService_ILookup extends grpc.MethodDefinition<management_pb.LookupRequest, management_pb.LookupResponse> {
    path: "/platune.management.v1.Management/Lookup";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<management_pb.LookupRequest>;
    requestDeserialize: grpc.deserialize<management_pb.LookupRequest>;
    responseSerialize: grpc.serialize<management_pb.LookupResponse>;
    responseDeserialize: grpc.deserialize<management_pb.LookupResponse>;
}
interface IManagementService_IGetSongByPath extends grpc.MethodDefinition<management_pb.PathMessage, management_pb.SongResponse> {
    path: "/platune.management.v1.Management/GetSongByPath";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<management_pb.PathMessage>;
    requestDeserialize: grpc.deserialize<management_pb.PathMessage>;
    responseSerialize: grpc.serialize<management_pb.SongResponse>;
    responseDeserialize: grpc.deserialize<management_pb.SongResponse>;
}
interface IManagementService_IGetAlbumsByAlbumArtists extends grpc.MethodDefinition<management_pb.IdMessage, management_pb.AlbumResponse> {
    path: "/platune.management.v1.Management/GetAlbumsByAlbumArtists";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<management_pb.IdMessage>;
    requestDeserialize: grpc.deserialize<management_pb.IdMessage>;
    responseSerialize: grpc.serialize<management_pb.AlbumResponse>;
    responseDeserialize: grpc.deserialize<management_pb.AlbumResponse>;
}
interface IManagementService_IGetDeleted extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, management_pb.GetDeletedResponse> {
    path: "/platune.management.v1.Management/GetDeleted";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<management_pb.GetDeletedResponse>;
    responseDeserialize: grpc.deserialize<management_pb.GetDeletedResponse>;
}
interface IManagementService_IDeleteTracks extends grpc.MethodDefinition<management_pb.IdMessage, google_protobuf_empty_pb.Empty> {
    path: "/platune.management.v1.Management/DeleteTracks";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<management_pb.IdMessage>;
    requestDeserialize: grpc.deserialize<management_pb.IdMessage>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementService_ISubscribeEvents extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, management_pb.Progress> {
    path: "/platune.management.v1.Management/SubscribeEvents";
    requestStream: false;
    responseStream: true;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<management_pb.Progress>;
    responseDeserialize: grpc.deserialize<management_pb.Progress>;
}

export const ManagementService: IManagementService;

export interface IManagementServer extends grpc.UntypedServiceImplementation {
    startSync: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty>;
    addFolders: grpc.handleUnaryCall<management_pb.FoldersMessage, google_protobuf_empty_pb.Empty>;
    getAllFolders: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, management_pb.FoldersMessage>;
    registerMount: grpc.handleUnaryCall<management_pb.RegisteredMountMessage, google_protobuf_empty_pb.Empty>;
    getRegisteredMount: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, management_pb.RegisteredMountMessage>;
    search: grpc.handleBidiStreamingCall<management_pb.SearchRequest, management_pb.SearchResponse>;
    lookup: grpc.handleUnaryCall<management_pb.LookupRequest, management_pb.LookupResponse>;
    getSongByPath: grpc.handleUnaryCall<management_pb.PathMessage, management_pb.SongResponse>;
    getAlbumsByAlbumArtists: grpc.handleUnaryCall<management_pb.IdMessage, management_pb.AlbumResponse>;
    getDeleted: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, management_pb.GetDeletedResponse>;
    deleteTracks: grpc.handleUnaryCall<management_pb.IdMessage, google_protobuf_empty_pb.Empty>;
    subscribeEvents: grpc.handleServerStreamingCall<google_protobuf_empty_pb.Empty, management_pb.Progress>;
}

export interface IManagementClient {
    startSync(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    startSync(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    startSync(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    addFolders(request: management_pb.FoldersMessage, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    addFolders(request: management_pb.FoldersMessage, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    addFolders(request: management_pb.FoldersMessage, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    getAllFolders(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_pb.FoldersMessage) => void): grpc.ClientUnaryCall;
    getAllFolders(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_pb.FoldersMessage) => void): grpc.ClientUnaryCall;
    getAllFolders(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_pb.FoldersMessage) => void): grpc.ClientUnaryCall;
    registerMount(request: management_pb.RegisteredMountMessage, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    registerMount(request: management_pb.RegisteredMountMessage, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    registerMount(request: management_pb.RegisteredMountMessage, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    getRegisteredMount(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_pb.RegisteredMountMessage) => void): grpc.ClientUnaryCall;
    getRegisteredMount(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_pb.RegisteredMountMessage) => void): grpc.ClientUnaryCall;
    getRegisteredMount(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_pb.RegisteredMountMessage) => void): grpc.ClientUnaryCall;
    search(): grpc.ClientDuplexStream<management_pb.SearchRequest, management_pb.SearchResponse>;
    search(options: Partial<grpc.CallOptions>): grpc.ClientDuplexStream<management_pb.SearchRequest, management_pb.SearchResponse>;
    search(metadata: grpc.Metadata, options?: Partial<grpc.CallOptions>): grpc.ClientDuplexStream<management_pb.SearchRequest, management_pb.SearchResponse>;
    lookup(request: management_pb.LookupRequest, callback: (error: grpc.ServiceError | null, response: management_pb.LookupResponse) => void): grpc.ClientUnaryCall;
    lookup(request: management_pb.LookupRequest, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_pb.LookupResponse) => void): grpc.ClientUnaryCall;
    lookup(request: management_pb.LookupRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_pb.LookupResponse) => void): grpc.ClientUnaryCall;
    getSongByPath(request: management_pb.PathMessage, callback: (error: grpc.ServiceError | null, response: management_pb.SongResponse) => void): grpc.ClientUnaryCall;
    getSongByPath(request: management_pb.PathMessage, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_pb.SongResponse) => void): grpc.ClientUnaryCall;
    getSongByPath(request: management_pb.PathMessage, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_pb.SongResponse) => void): grpc.ClientUnaryCall;
    getAlbumsByAlbumArtists(request: management_pb.IdMessage, callback: (error: grpc.ServiceError | null, response: management_pb.AlbumResponse) => void): grpc.ClientUnaryCall;
    getAlbumsByAlbumArtists(request: management_pb.IdMessage, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_pb.AlbumResponse) => void): grpc.ClientUnaryCall;
    getAlbumsByAlbumArtists(request: management_pb.IdMessage, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_pb.AlbumResponse) => void): grpc.ClientUnaryCall;
    getDeleted(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_pb.GetDeletedResponse) => void): grpc.ClientUnaryCall;
    getDeleted(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_pb.GetDeletedResponse) => void): grpc.ClientUnaryCall;
    getDeleted(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_pb.GetDeletedResponse) => void): grpc.ClientUnaryCall;
    deleteTracks(request: management_pb.IdMessage, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    deleteTracks(request: management_pb.IdMessage, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    deleteTracks(request: management_pb.IdMessage, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    subscribeEvents(request: google_protobuf_empty_pb.Empty, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<management_pb.Progress>;
    subscribeEvents(request: google_protobuf_empty_pb.Empty, metadata?: grpc.Metadata, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<management_pb.Progress>;
}

export class ManagementClient extends grpc.Client implements IManagementClient {
    constructor(address: string, credentials: grpc.ChannelCredentials, options?: Partial<grpc.ClientOptions>);
    public startSync(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public startSync(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public startSync(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public addFolders(request: management_pb.FoldersMessage, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public addFolders(request: management_pb.FoldersMessage, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public addFolders(request: management_pb.FoldersMessage, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public getAllFolders(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_pb.FoldersMessage) => void): grpc.ClientUnaryCall;
    public getAllFolders(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_pb.FoldersMessage) => void): grpc.ClientUnaryCall;
    public getAllFolders(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_pb.FoldersMessage) => void): grpc.ClientUnaryCall;
    public registerMount(request: management_pb.RegisteredMountMessage, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public registerMount(request: management_pb.RegisteredMountMessage, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public registerMount(request: management_pb.RegisteredMountMessage, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public getRegisteredMount(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_pb.RegisteredMountMessage) => void): grpc.ClientUnaryCall;
    public getRegisteredMount(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_pb.RegisteredMountMessage) => void): grpc.ClientUnaryCall;
    public getRegisteredMount(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_pb.RegisteredMountMessage) => void): grpc.ClientUnaryCall;
    public search(options?: Partial<grpc.CallOptions>): grpc.ClientDuplexStream<management_pb.SearchRequest, management_pb.SearchResponse>;
    public search(metadata?: grpc.Metadata, options?: Partial<grpc.CallOptions>): grpc.ClientDuplexStream<management_pb.SearchRequest, management_pb.SearchResponse>;
    public lookup(request: management_pb.LookupRequest, callback: (error: grpc.ServiceError | null, response: management_pb.LookupResponse) => void): grpc.ClientUnaryCall;
    public lookup(request: management_pb.LookupRequest, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_pb.LookupResponse) => void): grpc.ClientUnaryCall;
    public lookup(request: management_pb.LookupRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_pb.LookupResponse) => void): grpc.ClientUnaryCall;
    public getSongByPath(request: management_pb.PathMessage, callback: (error: grpc.ServiceError | null, response: management_pb.SongResponse) => void): grpc.ClientUnaryCall;
    public getSongByPath(request: management_pb.PathMessage, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_pb.SongResponse) => void): grpc.ClientUnaryCall;
    public getSongByPath(request: management_pb.PathMessage, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_pb.SongResponse) => void): grpc.ClientUnaryCall;
    public getAlbumsByAlbumArtists(request: management_pb.IdMessage, callback: (error: grpc.ServiceError | null, response: management_pb.AlbumResponse) => void): grpc.ClientUnaryCall;
    public getAlbumsByAlbumArtists(request: management_pb.IdMessage, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_pb.AlbumResponse) => void): grpc.ClientUnaryCall;
    public getAlbumsByAlbumArtists(request: management_pb.IdMessage, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_pb.AlbumResponse) => void): grpc.ClientUnaryCall;
    public getDeleted(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_pb.GetDeletedResponse) => void): grpc.ClientUnaryCall;
    public getDeleted(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_pb.GetDeletedResponse) => void): grpc.ClientUnaryCall;
    public getDeleted(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_pb.GetDeletedResponse) => void): grpc.ClientUnaryCall;
    public deleteTracks(request: management_pb.IdMessage, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public deleteTracks(request: management_pb.IdMessage, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public deleteTracks(request: management_pb.IdMessage, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public subscribeEvents(request: google_protobuf_empty_pb.Empty, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<management_pb.Progress>;
    public subscribeEvents(request: google_protobuf_empty_pb.Empty, metadata?: grpc.Metadata, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<management_pb.Progress>;
}
