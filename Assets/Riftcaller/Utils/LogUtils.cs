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

using System;
using UnityEngine;

namespace Riftcaller.Utils
{
  public static class LogUtils
  {
    public static void Log(string message)
    {
      message = $"[UNITY]  {message}";
#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
      Console.WriteLine(message);
#else      
      Debug.Log(message);
#endif       
    }
    
    public static void LogError(string message)
    { 
      message = $"[ERROR]  {message}";          
#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
      Console.Error.WriteLine(message);
#else      
      Debug.LogError(message);
#endif       
    }    
  }
}