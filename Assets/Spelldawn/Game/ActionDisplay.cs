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

using Spelldawn.Protos;
using Spelldawn.Utils;
using TMPro;
using UnityEngine;
using UnityEngine.Serialization;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class ActionDisplay : MonoBehaviour
  {
    [SerializeField] uint _availableActions = 3;
    [SerializeField] TextMeshProUGUI _number = null!;
    [FormerlySerializedAs("_left")] [SerializeField] ActionSymbol _one = null!;
    [FormerlySerializedAs("_center")] [SerializeField] ActionSymbol _two = null!;
    [FormerlySerializedAs("_right")] [SerializeField] ActionSymbol _three = null!;
    [SerializeField] ActionSymbol _four = null!;    

    public uint AvailableActions => _availableActions;

    public bool IsAnimating => _one.IsAnimating || _two.IsAnimating || _three.IsAnimating || _four.IsAnimating;

    public void DisableAnimation()
    {
      var disabled = new Material(Shader.Find("TextMeshPro/Distance Field"));
      _one.SetFontMaterial(disabled);
      _two.SetFontMaterial(disabled);
      _three.SetFontMaterial(disabled);
      _four.SetFontMaterial(disabled);
    }

    public void RenderActionTrackerView(ActionTrackerView actionTrackerView)
    {
      SetAvailableActions(actionTrackerView.AvailableActionCount);
      SetDefaultActionCount(actionTrackerView.DefaultActionCount);
    }

    public void SpendActions(uint amount)
    {
      Errors.CheckArgument(amount <= _availableActions, "Not enough actions available");
      SetAvailableActions(_availableActions - amount);
    }

    public void GainActions(uint amount)
    {
      SetAvailableActions(_availableActions + amount);
    }

    public void SetAvailableActions(uint availableActions)
    {
      _availableActions = availableActions;
      _number.gameObject.SetActive(false);

      switch (availableActions)
      {
        case 0:
          _one.SetFilled(false);
          _two.SetFilled(false);
          _three.SetFilled(false);
          _four.SetFilled(false);
          break;
        case 1:
          _one.SetFilled(false);
          _two.SetFilled(false);
          _three.SetFilled(false);
          _four.SetFilled(true);
          break;
        case 2:
          _one.SetFilled(false);
          _two.SetFilled(false);
          _three.SetFilled(true);
          _four.SetFilled(true);
          break;
        case 3:
          _one.SetFilled(false);
          _two.SetFilled(true);
          _three.SetFilled(true);
          _four.SetFilled(true);
          break;
        case 4:
          _one.SetFilled(true);
          _two.SetFilled(true);
          _three.SetFilled(true);
          _four.SetFilled(true);
          break;
        default:
          _one.gameObject.SetActive(false);
          _two.gameObject.SetActive(false);
          _three.SetFilled(true);
          _four.gameObject.SetActive(false);
          _number.gameObject.SetActive(true);
          _number.text = availableActions + "";
          break;
      }
    }

    public void SetDefaultActionCount(uint defaultActions)
    {
      switch (defaultActions)
      {
        case 1:
          _three.gameObject.SetActive(false);
          goto case 2;        
        case 2:
          _two.gameObject.SetActive(false);
          goto case 3;
        case 3:
          _one.gameObject.SetActive(false);
          break;
      }
    }
  }
}