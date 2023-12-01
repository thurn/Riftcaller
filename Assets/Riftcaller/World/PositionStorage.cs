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

using Riftcaller.Services;
using Riftcaller.Utils;
using UnityEngine;

namespace Riftcaller.World
{
  public static class PositionStorage
  {
    public static void StorePosition(Registry registry, Vector3Int position)
    {
      var adventureId = Errors.CheckNotNull(registry.CommandService.ClientMetadata.AdventureId, "Expected AdventureId");
      PlayerPrefs.SetString(Preferences.WorldPosition, $"{adventureId}/{position.x}/{position.y}/{position.z}");
      PlayerPrefs.Save();
    }

    public static Vector3Int? GetPosition(Registry registry)
    {
      var adventureId = Errors.CheckNotNull(registry.CommandService.ClientMetadata.AdventureId, "Expected AdventureId");
      
      if (!PlayerPrefs.HasKey(Preferences.WorldPosition))
      {
        return null;
      }
      
      var values = PlayerPrefs.GetString(Preferences.WorldPosition).Split("/");
      if (values[0] == adventureId)
      {
        return new Vector3Int(int.Parse(values[1]), int.Parse(values[2]), int.Parse(values[3]));
      }
      else
      {
        return null;
      }
    }
  }
}