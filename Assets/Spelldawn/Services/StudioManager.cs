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
using Spelldawn.Game;
using Spelldawn.Protos;
using Spelldawn.Utils;
using UnityEngine;
using UnityEngine.UIElements;

namespace Spelldawn.Services
{
  public sealed class StudioManager : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] Studio _studioPrefab = null!;
    readonly Dictionary<StudioDisplay, Studio> _studios = new();
    int _nextX = 2;
    int _nextStudioNumber = 1;

    IEnumerator Start()
    {
      while (true)
      {
        // Periodically remove studios which are not currently being used in the UI.
        var remove = _studios.Where(kvp =>
          !_registry.DocumentService.RootVisualElement.Query().Class(kvp.Value.ClassNameTag()).Build().Any()).ToList();
        foreach (var (key, studio) in remove)
        {
          _studios.Remove(key);
          Destroy(studio.gameObject);
        }

        yield return new WaitForSeconds(1.0f);
      }
    }    
    
    public void DisplayAsBackground(VisualElement element, StudioDisplay display)
    {
      StartCoroutine(DisplayAsBackgroundAsync(element, display));
    }

    IEnumerator DisplayAsBackgroundAsync(VisualElement element, StudioDisplay display)
    {
      if (_studios.ContainsKey(display))
      {
        SetStudio(element, _studios[display]);
        yield break;
      }

      yield return _registry.AssetService.LoadStudioAssets(display);

      var subject = display.DisplayCase switch
      {
        StudioDisplay.DisplayOneofCase.Card =>
          _registry.CardService.CreateCard(display.Card, GameContext.InfoZoom, animate: false).gameObject,
        _ => throw new ArgumentOutOfRangeException()
      };

      var studio = ComponentUtils.Instantiate(_studioPrefab);
      studio.Initialize(_nextStudioNumber++);
      studio.transform.position = new Vector3(_nextX++ * 50, 50, 50);
      studio.SetSubject(subject);
      SetStudio(element, studio);
      _studios[display] = studio;
    }

    void SetStudio(VisualElement element, Studio studio)
    {
      if (!element.ClassListContains(studio.ClassNameTag()))
      {
        element.AddToClassList(studio.ClassNameTag());
      }
      element.style.backgroundImage = new StyleBackground(new Background { renderTexture = studio.RenderTexture });
    }
  }
}