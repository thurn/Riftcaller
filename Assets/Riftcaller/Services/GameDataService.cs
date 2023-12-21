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

using System;
using System.Collections;
using Riftcaller.Protos;
using Riftcaller.Utils;
using UnityEngine;

namespace Riftcaller.Services
{
  [Serializable]
  public sealed class GameTable
  {
    [SerializeField] GameTableName _tableName;
    public GameTableName GameTableName => _tableName;
    [SerializeField] TextAsset _asset = null!;
    public TextAsset TextAsset => _asset;
  }
  
  public sealed class GameDataService: MonoBehaviour
  {
    [SerializeField] GameTable[] _tables = null!;

    public IEnumerator Initialize(Registry registry)
    {
      // This currently runs every time a scene loads, could think about
      // making it only happen on game start
      foreach (var table in _tables)
      {
        if (registry.ActionService.DevelopmentMode())
        {
          yield return registry.ActionService.SetGameTable(new SetGameTableRequest()
          {
            TableName = table.GameTableName,
            FileContent = table.TextAsset.text
          });
        }
        else
        {
          Plugin.SetGameTable(table.GameTableName, table.TextAsset.text);
        }
      }
    }
  }
}