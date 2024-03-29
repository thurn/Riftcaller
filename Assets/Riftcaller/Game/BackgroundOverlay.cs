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

using System.Collections.Generic;
using System.Linq;
using DG.Tweening;
using UnityEngine;

#nullable enable

namespace Riftcaller.Game
{
  public sealed class BackgroundOverlay : MonoBehaviour
  {
    [SerializeField] GameContext _gameContext;
    [SerializeField] SpriteRenderer _renderer = null!;
    [SerializeField] bool _forceEnabled;
    readonly List<ObjectDisplay> _displays = new();

    public bool Enabled => _renderer.enabled;

    public void AddObjectDisplay(ObjectDisplay display)
    {
      _displays.Add(display);
    }

    void Update()
    {
      if (_forceEnabled)
      {
        return;
      }
      
      if (_displays.Any(d => d.AllObjects.Count > 0))
      {
        Enable(true);
      }
      else
      {
        Disable();
      }
    }

    public void ForceEnable()
    {
      _forceEnabled = true;
      Enable(translucent: false);
    }

    void Enable(bool translucent)
    {
      if (!Enabled)
      {
        SortingOrder.Create(_gameContext, -1).ApplyTo(_renderer);
        _renderer.enabled = true;
        _renderer.color = Color.clear;
        _renderer.DOBlendableColor(translucent ? new Color(0, 0, 0, 0.5f) : Color.black, 0.3f);
      }
    }

    void Disable()
    {
      _renderer.enabled = false;
    }
  }
}