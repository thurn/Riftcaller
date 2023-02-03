// Copyright © Spelldawn 2021-present

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

using System.Collections.Generic;
using System.Linq;
using Spelldawn.Protos;
using UnityEngine;

namespace Spelldawn.Services
{
  public sealed class AnalyticsService : MonoBehaviour
  {
    readonly Dictionary<string, string> _metadata = new();

    public string CurrentMetadata => string.Join(", ", _metadata.Select(pair => $"[{pair.Key}: {pair.Value}]")); 

    public void SetMetadata(IEnumerable<LoggingMetadata> metadata)
    {
      foreach (var data in metadata)
      {
        if (!_metadata.ContainsKey(data.Key) || _metadata[data.Key] != data.Value)
        {
          _metadata[data.Key] = data.Value;
          
          // This feature only actually works if you pay Unity a bunch of money >:(
          UnityEngine.CrashReportHandler.CrashReportHandler.SetUserMetadata(data.Key, data.Value);
        }
      }
    }
  }
}