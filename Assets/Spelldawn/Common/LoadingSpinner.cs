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
using UnityEngine;
using UnityEngine.UIElements;

namespace Spelldawn.Common
{
  public enum WaitingFor
  {
    Connection,
    PanelFetch,
    FeedbackForm,
    Assets
  }
  
  public sealed class LoadingSpinner
  {
    readonly HashSet<WaitingFor> _currentlyWaitingFor = new();
    Image? _loading;
    Label? _label;
    float _rotateAngle;
    
    public void Initialize(VisualElement loadingContainer, Sprite loadingSprite)
    {
      loadingContainer.style.justifyContent = Justify.Center;
      loadingContainer.style.alignItems = Align.Center;
      _loading = new Image
      {
        sprite = loadingSprite,
        style =
        {
          width = 88,
          height = 88,
          opacity = 0.5f
        }
      };
      loadingContainer.Add(_loading);
      _label = new Label
      {
        style =
        {
          fontSize = 24,
          color = Color.white,
          backgroundColor = new Color(0f, 0f, 0f, 0.7f),
          left = 8 + Screen.safeArea.xMin,
          bottom = 8 + Screen.safeArea.yMin,
          borderBottomLeftRadius = 8,
          borderBottomRightRadius = 8,
          borderTopLeftRadius = 8,
          borderTopRightRadius = 8,
          paddingLeft = 8,
          paddingRight = 8,
          paddingTop = 8,
          paddingBottom = 8,
          position = Position.Absolute
        }
      };
      loadingContainer.Add(_label);
    }

    public void WaitFor(WaitingFor waitingFor)
    {
      _currentlyWaitingFor.Add(waitingFor);
    }

    public void EndWaitFor(WaitingFor waitingFor)
    {
      _currentlyWaitingFor.Remove(waitingFor);
    }

    public void Update()
    {
      if (_loading != null && _label != null && _currentlyWaitingFor.Count > 0)
      {
        _loading.visible = true;
        _label.visible = true;
        _rotateAngle = (_rotateAngle + (Time.deltaTime * 600)) % 360;
        _loading.style.rotate = new Rotate(Angle.Degrees(_rotateAngle));
        var waitingFor = string.Join(", ", _currentlyWaitingFor);
        _label.text = $"Waiting for {waitingFor}";
      }
      else if (_loading != null && _label != null)
      {
        _loading.visible = false;
        _label.visible = false;        
      }
    }
  }
}