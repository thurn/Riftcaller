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

#nullable enable

using System.Runtime.InteropServices;
using System.Text;
using Google.Protobuf;
using Riftcaller.Protos;
using UnityEngine;

namespace Riftcaller.Utils
{
  static class Plugin
  {
    const int BufferSize = 1_000_000;
    static readonly byte[] PollBuffer = new byte[BufferSize];
    static bool _initialized;

    public static CommandList? Connect(ConnectRequest request)
    {
      if (!_initialized)
      {
        var path = $"{Application.persistentDataPath}/db";
        var encoded = Encoding.UTF8.GetBytes(path);
        Errors.CheckNonNegative(riftcaller_initialize(encoded, encoded.Length), "Plugin initialization error");
        _initialized = true;
      }

      var input = request.ToByteArray();
      var output = new byte[BufferSize];
      var responseSize = Errors.CheckNonNegative(
          riftcaller_connect(input, input.Length, output, output.Length),
          "Plugin connect error");
      return responseSize > 0 ? CommandList.Parser.ParseFrom(output, 0, responseSize) : null;
    }

    public static CommandList? Poll(PollRequest request)
    {
      var input = request.ToByteArray();
      var responseSize = Errors.CheckNonNegative(
          riftcaller_poll(input, input.Length, PollBuffer, PollBuffer.Length),
          "Plugin poll error");
      return responseSize > 0 ? CommandList.Parser.ParseFrom(PollBuffer, 0, responseSize) : null;
    }

    public static CommandList PerformAction(GameRequest request)
    {
      var input = request.ToByteArray();
      var output = new byte[BufferSize];
      var responseSize = Errors.CheckNonNegative(
          riftcaller_perform_action(input, input.Length, output, output.Length),
          "Plugin action error");
      return CommandList.Parser.ParseFrom(output, 0, responseSize);
    }

#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
    [DllImport("__Internal")]
#else
    [DllImport("plugin")]
#endif
    public static extern int riftcaller_initialize(byte[] path, int pathLength);

#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
    [DllImport("__Internal")]
#else
    [DllImport("plugin")]
#endif
    public static extern int riftcaller_connect(
      byte[] request,
      int requestLength,
      [Out] byte[] response,
      int responseLength);

#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
    [DllImport("__Internal")]
#else
    [DllImport("plugin")]
#endif
    public static extern int riftcaller_poll(
      byte[] request,
      int requestLength,
      [Out] byte[] response,
      int responseLength);

#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
    [DllImport("__Internal")]
#else
    [DllImport("plugin")]
#endif
    public static extern int riftcaller_perform_action(
      byte[] request,
      int requestLength,
      [Out] byte[] response,
      int responseLength);
  }
}
