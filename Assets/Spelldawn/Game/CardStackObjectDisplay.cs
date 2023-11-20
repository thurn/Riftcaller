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
using Spelldawn.Utils;
using UnityEngine;

namespace Spelldawn.Game
{
  public sealed class CardStackObjectDisplay : MonoBehaviour, IObjectDisplay
  {
    public List<Displayable> _allObjects = null!;

    public List<Displayable> AllObjects => new(_allObjects);
    
    public MonoBehaviour AsMonoBehaviour() => this;

    public IEnumerator AddObject(Displayable displayable, bool animate = true)
    {
      AddObjectImmediate(displayable);
      yield break;
    }

    public void AddObjectImmediate(Displayable displayable)
    {
      if (!_allObjects.Contains(displayable))
      {
        _allObjects.Add(displayable);
        if (displayable.Parent != null && displayable.Parent.AsMonoBehaviour())
        {
          displayable.Parent.RemoveObjectIfPresent(displayable, false);
        }        
      }

      displayable.transform.SetParent(transform);
      displayable.transform.localPosition = Vector3.zero;
      displayable.Parent = this;
      displayable.SetGameContext(GameContext.BehindArena);      
    }

    public void RemoveObject(Displayable displayable, bool animate = true)
    {
      var index = _allObjects.FindIndex(c => c == displayable);
      Errors.CheckNonNegative(index);
      _allObjects.RemoveAt(index);
      displayable.Parent = null;
      displayable.transform.SetParent(null);
    }

    public void RemoveObjectIfPresent(Displayable displayable, bool animate = true)
    {
      if (_allObjects.Contains(displayable))
      {
        RemoveObject(displayable, animate);
      }
    }
  }
}