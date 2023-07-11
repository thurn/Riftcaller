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
using System.Collections.Generic;
using System.Linq;
using DG.Tweening;
using Spelldawn.Masonry;
using Spelldawn.Protos;
using Spelldawn.Services;
using Spelldawn.Utils;
using UnityEngine;
using UnityEngine.Tilemaps;

namespace Spelldawn.World
{
  public sealed class WorldMap : MonoBehaviour, Dijkstra<Vector3Int>.IGraph
  {
    static readonly Vector3Int Left = new(-1, 0, 0);
    static readonly Vector3Int Right = new(1, 0, 0);
    static readonly Vector3Int Down = new(0, -1, 0);
    static readonly Vector3Int Downleft = new(-1, -1, 0);
    static readonly Vector3Int Downright = new(1, -1, 0);
    static readonly Vector3Int Up = new(0, 1, 0);
    static readonly Vector3Int Upleft = new(-1, 1, 0);
    static readonly Vector3Int Upright = new(1, 1, 0);
    static readonly Vector3Int[] DirectionsWhenYIsEven = { Left, Right, Down, Downleft, Up, Upleft };
    static readonly Vector3Int[] DirectionsWhenYIsOdd = { Left, Right, Down, Downright, Up, Upright };

    [SerializeField] Registry _registry = null!;
    [SerializeField] Tilemap _tilemapPrefab = null!;
    [SerializeField] GameObject _selectedHex = null!;

    public sealed record TileId(MapPosition Position, int Z);

    readonly Dictionary<TileId, Tilemap> _tilemaps = new();
    readonly Dictionary<MapPosition, WorldMapTile> _tiles = new();
    static readonly TileId TileZero = new(new MapPosition(), 0);

    void Start()
    {
      // Always create at least one tilemap to use for resolving positions
      GetTilemap(TileZero);
    }

    public IEnumerator HandleUpdateWorldMap(UpdateWorldMapCommand command)
    {
      _registry.CharacterService.InitializeIfNeeded();
      
      foreach (var tilemap in _tilemaps.Values)
      {
        tilemap.ClearAllTiles();
      }
      
      foreach (var tile in command.Tiles)
      {
        var zIndex = 0;
        foreach (var sprite in tile.Sprites)
        {
          // Unity has 3 different ways of managing tile sort order (global sort axis, tilemap z position,
          // and sprite sorting order). In my experience these layers are generally buggy and unreliable,
          // especially tilemap z-index. To make sure everything works as expected, we *only* rely on 
          // sprite sorting order and instantiate numerous different Tilemaps with different sorting
          // behavior.

          var instance = ScriptableObject.CreateInstance<Tile>();
          instance.sprite = _registry.AssetService.GetSprite(sprite.SpriteAddress);

          var matrix = Matrix4x4.identity;
          if (sprite.AnchorOffset != null)
          {
            matrix *= Matrix4x4.Translate(new Vector3(sprite.AnchorOffset.X, sprite.AnchorOffset.Y,
              sprite.AnchorOffset.Z));
          }

          if (sprite.Scale != null)
          {
            matrix *= Matrix4x4.Scale(new Vector3(sprite.Scale.X, sprite.Scale.Y, sprite.Scale.Z));
          }

          if (sprite.Color != null)
          {
            instance.color = Mason.ToUnityColor(sprite.Color);
          }

          instance.transform = matrix;
          var id = new TileId(tile.Position, zIndex++);
          GetTilemap(id).SetTile(new Vector3Int(tile.Position.X, tile.Position.Y, 0), instance);
        }

        if (tile.Character != null)
        {
          _registry.CharacterService.CreateOrUpdateCharacter(tile.Position, tile.Character);
        }
        _tiles[tile.Position] = tile;
      }

      yield break;
    }

    public void OnClick(Vector3 position)
    {
      var cellPosition = _tilemaps[TileZero].layoutGrid.WorldToCell(new Vector3(position.x, position.y, 0));
      var mapPosition = FromVector3Int(cellPosition);
      var tile = _tiles[mapPosition];
      var tileType = tile.TileType;

      if (tileType is MapTileType.Walkable or MapTileType.Visitable)
      {
        var pos = _tilemaps[TileZero].CellToWorld(cellPosition);
        var selected = ComponentUtils.InstantiateGameObject(_selectedHex);
        selected.transform.position = pos;
        selected.GetComponent<SpriteRenderer>().sortingOrder = SortOrderForTileId(new TileId(mapPosition, 5));

        TweenUtils.Sequence("HexClick")
          .Append(selected.transform.DOScale(Vector3.zero, 0.5f).SetEase(Ease.InExpo))
          .AppendCallback(() => Destroy(selected));

        var start = _registry.CharacterService.CurrentHeroPosition();

        var path = tileType is MapTileType.Visitable
          ? Dijkstra<Vector3Int>.ShortestPathOfDestinations(this, start, FindNeighbors(cellPosition))
          : Dijkstra<Vector3Int>.ShortestPath(this, start, cellPosition);
        if (path.Count > 0)
        {
          PositionStorage.StorePosition(_registry, path.Last());
        }

        var worldPath = path.Select(v => ToWorldPosition(new MapPosition
        {
          X = v.x,
          Y = v.y
        })).ToList();
        _registry.CharacterService.MoveHero(worldPath,
          tile.OnVisit == null ? null : () => { _registry.ActionService.HandleAction(tile.OnVisit); });
      }
    }

    public List<Vector3Int> FindNeighbors(Vector3Int vertex)
    {
      var result = new List<Vector3Int>();
      var directions = (vertex.y % 2) == 0 ? DirectionsWhenYIsEven : DirectionsWhenYIsOdd;

      foreach (var direction in directions)
      {
        var neighborPos = vertex + direction;
        if (_tiles.GetValueOrDefault(FromVector3Int(neighborPos))?.TileType == MapTileType.Walkable)
        {
          result.Add(neighborPos);
        }
      }

      return result;
    }

    public List<Vector3Int> Vertices() => _tiles.Values.Where(t => t.TileType == MapTileType.Walkable)
      .Select(t => ToVector3Int(t.Position)).ToList();

    MapPosition FromVector3Int(Vector3Int vector) => new() { X = vector.x, Y = vector.y };

    Vector3Int ToVector3Int(MapPosition position, int z = 0) => new() { x = position.X, y = position.Y, z = z };

    public Vector3 ToWorldPosition(Vector3Int vector) => ToWorldPosition(FromVector3Int(vector));
    
    public Vector3 ToWorldPosition(MapPosition mapPosition) =>
      GetTilemap(new TileId(mapPosition, 0)).layoutGrid.CellToWorld(new Vector3Int(mapPosition.X, mapPosition.Y, 0));

    public MapPosition FromWorldPosition(Vector3 worldPosition)
    {
      var position = _tilemaps[TileZero].layoutGrid.WorldToCell(worldPosition);
      return new MapPosition
      {
        X = position.x,
        Y = position.y
      };
    }

    public int SortOrderForTileId(TileId tileId) => (tileId.Position.Y * -100) + tileId.Z;

    Tilemap GetTilemap(TileId tileId)
    {
      if (!_tilemaps.ContainsKey(tileId))
      {
        var tilemap = ComponentUtils.Instantiate(_tilemapPrefab);
        tilemap.transform.SetParent(transform);
        tilemap.transform.position = Vector3.zero;
        ComponentUtils.GetComponent<TilemapRenderer>(tilemap).sortingOrder = SortOrderForTileId(tileId);
        _tilemaps[tileId] = tilemap;
        return tilemap;
      }
      else
      {
        return _tilemaps[tileId];
      }
    }
  }
}