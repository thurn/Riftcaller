// Copyright © Riftcaller 2021-present

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//    https://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(clippy::missing_safety_doc)] // You only live once, that's the motto - Drake

//! Implements a DLL for Unity to call into the Riftcaller API

use std::panic::UnwindSafe;
use std::{panic, str};

use anyhow::Result;
use database::sled_database::SledDatabase;
use logging::LoggingType;
use once_cell::sync::Lazy;
use prost::Message;
use protos::riftcaller::client_debug_command::DebugCommand;
use protos::riftcaller::game_command::Command;
use protos::riftcaller::{
    ClientDebugCommand, CommandList, ConnectRequest, GameCommand, GameRequest, LogMessage,
    LogMessageLevel, PollRequest,
};
use tokio::sync::Mutex;

static DATABASE: Lazy<Mutex<Option<SledDatabase>>> = Lazy::new(|| Mutex::new(None));

/// Initialize the plugin. Must be called immediately at application start.
///
/// Should be invoked with a buffer containing a UTF-8 encoded string of the
/// database path the plugin should use, along with its length.
///
/// Returns 0 on success and -1 on error.
#[no_mangle]
pub unsafe extern "C" fn riftcaller_initialize(path: *const u8, path_length: i32) -> i32 {
    panic::catch_unwind(|| initialize_impl(path, path_length).unwrap_or(-1)).unwrap_or(-1)
}

#[tokio::main]
async unsafe fn initialize_impl(path: *const u8, path_length: i32) -> Result<i32> {
    cards_all::initialize();
    let slice = std::slice::from_raw_parts(path, path_length as usize);
    let db_path = str::from_utf8(slice)?;
    let mut guard = DATABASE.lock().await;
    let _ = guard.insert(SledDatabase::new(db_path));

    logging::initialize(LoggingType::Forest);
    println!("Initialized plugin with database path {db_path}");
    Ok(0)
}

/// Synchronize the state of an ongoing game, downloading a full description of
/// the game state.
///
/// `request` should be a buffer including the protobuf serialization of a
/// `ConnectRequest` message of `request_length` bytes. `response` should be an
/// empty buffer of `response_length` bytes, this buffer will be populated with
/// a protobuf-serialized `CommandList` describing the current state of the
/// game.
///
/// Returns the number of bytes written to the `response` buffer, or -1 on
/// error.
#[no_mangle]
pub unsafe extern "C" fn riftcaller_connect(
    request: *const u8,
    request_length: i32,
    response: *mut u8,
    response_length: i32,
) -> i32 {
    error_boundary(response, response_length, || {
        connect_impl(request, request_length, response, response_length)
    })
}

#[tokio::main]
async unsafe fn connect_impl(
    request: *const u8,
    request_length: i32,
    response: *mut u8,
    response_length: i32,
) -> Result<i32> {
    let request_data = std::slice::from_raw_parts(request, request_length as usize);
    let guard = DATABASE.lock().await;
    let database = guard.as_ref().expect("Database not initialized");
    let connect_request = ConnectRequest::decode(request_data)?;
    let player_id = server::parse_client_id(connect_request.player_id.as_ref())?;
    let game_response = server::plugin_connect(database, player_id).await?;
    let command_list = game_response.build().user_response;
    let mut out = std::slice::from_raw_parts_mut(response, response_length as usize);
    command_list.encode(&mut out)?;
    Ok(command_list.encoded_len() as i32)
}

/// Checks for new game responses which are available to be rendered on the
/// client.
///
/// `request` should be a buffer including the protobuf serialization of a
/// `PollRequest` message of `request_length` bytes.
///
/// `response` should be an empty buffer of `response_length` bytes, this buffer
/// will be populated with a protobuf-serialized `CommandList` describing an
/// update to the game state, if any is available.
///
/// Returns the number of bytes written to the `response` buffer, 0 if no update
/// is available, or -1 on error.
#[no_mangle]
pub unsafe extern "C" fn riftcaller_poll(
    request: *const u8,
    request_length: i32,
    response: *mut u8,
    response_length: i32,
) -> i32 {
    error_boundary(response, response_length, || {
        poll_impl(request, request_length, response, response_length)
    })
}

unsafe fn poll_impl(
    request: *const u8,
    request_length: i32,
    response: *mut u8,
    response_length: i32,
) -> Result<i32> {
    let poll_data = std::slice::from_raw_parts(request, request_length as usize);
    let poll_request = PollRequest::decode(poll_data)?;
    let player_id = server::parse_client_id(poll_request.player_id.as_ref())?;
    if let Some(command_list) = server::plugin_poll(player_id)? {
        let mut out = std::slice::from_raw_parts_mut(response, response_length as usize);
        command_list.encode(&mut out)?;
        Ok(command_list.encoded_len() as i32)
    } else {
        Ok(0)
    }
}

/// Performs a given game action.
///
/// `request` should be a buffer including the protobuf serialization of a
/// `GameRequest` message of `request_length` bytes. `response` should be an
/// empty buffer of `response_length` bytes, this buffer will be populated with
/// a protobuf-serialized `CommandList` describing the result of performing this
/// action.
///
/// Returns the number of bytes written to the `response` buffer, or -1 on
/// error.
#[no_mangle]
pub unsafe extern "C" fn riftcaller_perform_action(
    request: *const u8,
    request_length: i32,
    response: *mut u8,
    response_length: i32,
) -> i32 {
    error_boundary(response, response_length, || {
        perform_impl(request, request_length, response, response_length)
    })
}

#[tokio::main]
async unsafe fn perform_impl(
    request: *const u8,
    request_length: i32,
    response: *mut u8,
    response_length: i32,
) -> Result<i32> {
    let request_data = std::slice::from_raw_parts(request, request_length as usize);
    let game_request = GameRequest::decode(request_data)?;
    let guard = DATABASE.lock().await;
    let database = guard.as_ref().expect("Database not initialized");
    let player_id = server::parse_client_id(game_request.player_id.as_ref())?;
    let game_response = server::handle_action(database, player_id, &game_request).await?;
    let command_list = game_response.build().user_response;
    let mut out = std::slice::from_raw_parts_mut(response, response_length as usize);
    command_list.encode(&mut out)?;
    Ok(command_list.encoded_len() as i32)
}

unsafe fn error_boundary(
    response: *mut u8,
    response_length: i32,
    function: impl FnOnce() -> Result<i32> + UnwindSafe,
) -> i32 {
    panic::catch_unwind(|| match function() {
        Ok(i) => i,
        Err(e) => {
            let error = CommandList {
                logging_metadata: vec![],
                metadata: None,
                commands: vec![GameCommand {
                    command: Some(Command::Debug(ClientDebugCommand {
                        debug_command: Some(DebugCommand::LogMessage(LogMessage {
                            text: format!("{e:?}"),
                            level: LogMessageLevel::Error.into(),
                        })),
                    })),
                }],
            };

            let mut out = std::slice::from_raw_parts_mut(response, response_length as usize);
            error.encode(&mut out).expect("Error serializing error");
            error.encoded_len() as i32
        }
    })
    .unwrap_or(-1)
}

// Note: I figured out how to do function callbacks in the plugin, but I don't
// really need them right now. I'm writing it down here in case it comes up in
// the future and because I'm annoyed I wasted a bunch of time figuring this
// out.
//
// Basically you need to make a rust function like this:
//
// #[no_mangle]
// pub unsafe extern "C" fn riftcaller_callback(
//     callback: unsafe extern "C" fn(i32)) {}
//
// And then you make a delegate in C# like this:
// [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
// public delegate void CallbackDelegate(int writtenBytes);
//
// public static extern void riftcaller_callback(CallbackDelegate callback);
//
// The callback can only be a static function like this:
//     [MonoPInvokeCallback(typeof(CallbackDelegate))]
//     public static void CBack(int writtenBytes) {}
//
