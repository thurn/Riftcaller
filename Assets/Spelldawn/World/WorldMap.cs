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

using System.Collections;
using Spelldawn.Services;
using UnityEngine;
using UnityEngine.AddressableAssets;
using UnityEngine.Tilemaps;

namespace Spelldawn.World
{
  public sealed class WorldMap : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] Tilemap _worldTilemap = null!;

    IEnumerator Start()
    {
      var operation = Addressables.LoadAssetAsync<Sprite>("DavidBaumgart/WorldTiles.spriteatlas[hexPlainsColdSnowCovered01]");
      yield return operation;
      var tile = ScriptableObject.CreateInstance<Tile>();
      tile.sprite = operation.Result;
      _worldTilemap.SetTile(new Vector3Int(1, 1, 0), tile);
      
      var operation2 = Addressables.LoadAssetAsync<Sprite>("DavidBaumgart/Roads.spriteatlas[hexRoad-101000-00]");
      yield return operation2;
      var tile2 = ScriptableObject.CreateInstance<Tile>();
      tile2.sprite = operation2.Result;
      Debug.Log($"Start: got sprite: {operation2.Result.name} setting to tile {tile2.name}");
      _worldTilemap.SetTile(new Vector3Int(1, 1, 1), tile2);      
    }
  }
}