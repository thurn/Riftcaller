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

using System;
using System.Collections.Generic;
using Riftcaller.Utils;
using UnityEngine;
using UnityEngine.Rendering;

#nullable enable

namespace Riftcaller.Game
{
  public sealed class SortingOrder
  {
    public static SortingOrder Create(GameContext gameContext, int key = 0, int subkey = 0) =>
      new(gameContext, key, subkey);

    readonly GameContext _gameContext;
    readonly int _key;
    readonly int _subkey;
    readonly Lazy<Dictionary<GameContext, int>> _sortingLayers = new(() =>
    {
      var result = new Dictionary<GameContext, int>();
      foreach (GameContext context in Enum.GetValues(typeof(GameContext)))
      {
        result.Add(context, SortingLayer.NameToID(context.ToString()));
      }      
      return result;
    });

    SortingOrder(GameContext gameContext, int key, int subkey)
    {
      _gameContext = gameContext;
      _key = key;
      _subkey = subkey;
    }

    public void ApplyTo(SortingGroup group)
    {
      var position = Position();
      group.sortingOrder = position;
      group.sortingLayerID = _sortingLayers.Value[_gameContext];
    }

    public void ApplyTo(Renderer renderer)
    {
      var position = Position();
      renderer.sortingOrder = position;
      renderer.sortingLayerID = _sortingLayers.Value[_gameContext];
    }

    int Position()
    {
      var position = _subkey + (_key * 100);
      // Unity inexplicably uses 'int' for this type despite only allowing sorting keys below 32k
      Errors.CheckState(position < 32767, "Position overflow");
      return position;
    }
  }
}