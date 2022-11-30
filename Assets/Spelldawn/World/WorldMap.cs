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
using System.Collections.Generic;
using System.Linq;
using DG.Tweening;
using Spelldawn.Protos;
using Spelldawn.Services;
using Spelldawn.Utils;
using UnityEngine;
using UnityEngine.Tilemaps;

namespace Spelldawn.World
{
  public sealed class WorldMap : MonoBehaviour, Dijkstra<Vector3Int>.IGraph
  {
    static Vector3Int
      LEFT = new Vector3Int(-1, 0, 0),
      RIGHT = new Vector3Int(1, 0, 0),
      DOWN = new Vector3Int(0, -1, 0),
      DOWNLEFT = new Vector3Int(-1, -1, 0),
      DOWNRIGHT = new Vector3Int(1, -1, 0),
      UP = new Vector3Int(0, 1, 0),
      UPLEFT = new Vector3Int(-1, 1, 0),
      UPRIGHT = new Vector3Int(1, 1, 0);

    static Vector3Int[] directions_when_y_is_even =
      { LEFT, RIGHT, DOWN, DOWNLEFT, UP, UPLEFT };

    static Vector3Int[] directions_when_y_is_odd =
      { LEFT, RIGHT, DOWN, DOWNRIGHT, UP, UPRIGHT };

    [SerializeField] Registry _registry = null!;
    [SerializeField] Tilemap _worldTilemap = null!;
    [SerializeField] GameObject _selectedHex = null!;

    public IEnumerator HandleUpdateWorldMap(UpdateWorldMapCommand command)
    {
      foreach (var tile in command.Tiles)
      {
        var instance = ScriptableObject.CreateInstance<Tile>();
        instance.sprite = _registry.AssetService.GetSprite(tile.SpriteAddress);
        _worldTilemap.SetTile(new Vector3Int(tile.Position.X, tile.Position.Y, tile.ZIndex), instance);
      }

      yield break;
    }

    public Vector3 ToTilePosition(WorldPosition worldPosition) =>
      _worldTilemap.layoutGrid.CellToWorld(new Vector3Int(worldPosition.X, worldPosition.Y, 0));

    /// <summary>
    /// Character positions are offset from actual world positions in order to produce the correct sprite ordering.
    /// </summary>
    public Vector3 ToCharacterPosition(WorldPosition worldPosition)
    {
      var result = ToTilePosition(worldPosition);
      result.y -= 2.25f;
      return result;
    }

    public void OnClick(Vector3 position)
    {
      var cellPosition = _worldTilemap.layoutGrid.WorldToCell(new Vector3(position.x, position.y, 0));
      var pos = _worldTilemap.CellToWorld(cellPosition);
      var selected = ComponentUtils.InstantiateGameObject(_selectedHex);
      selected.transform.position = pos;

      TweenUtils.Sequence("HexClick")
        .Append(selected.transform.DOScale(Vector3.zero, 0.5f).SetEase(Ease.InBack))
        .AppendCallback(() => Destroy(selected));

      Debug.Log($"Getting shortest paths for {cellPosition}");
      var path = Dijkstra<Vector3Int>.ShortestPath(this, new Vector3Int(0, 0, 0), cellPosition);
      foreach (var vertex in path)
      {
        Debug.Log($"Shortest path: {vertex}");
      }
    }

    public List<Vector3Int> FindNeighbors(Vector3Int vertex)
    {
      var result = new List<Vector3Int>();
      Vector3Int[] directions = (vertex.y % 2) == 0 ? directions_when_y_is_even : directions_when_y_is_odd;
      foreach (var direction in directions)
      {
        Vector3Int neighborPos = vertex + direction;

        if (neighborPos.x >= _worldTilemap.cellBounds.min.x && neighborPos.x < _worldTilemap.cellBounds.max.x &&
            neighborPos.y >= _worldTilemap.cellBounds.min.y && neighborPos.y < _worldTilemap.cellBounds.max.y)
        {
          result.Add(neighborPos);
        }
      }

      return result;
    }

    public List<Vector3Int> Vertices()
    {
      var result = new List<Vector3Int>();
      for (var x = _worldTilemap.cellBounds.min.x; x < _worldTilemap.cellBounds.max.x; x++)
      {
        for (var y = _worldTilemap.cellBounds.min.y; y < _worldTilemap.cellBounds.max.y; y++)
        {
          result.Add(new Vector3Int(x, y, 0));
        }
      }

      return result;
    }
  }
}