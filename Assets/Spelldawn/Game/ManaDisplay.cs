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
using Spelldawn.Services;
using Spelldawn.Utils;
using TMPro;
using UnityEngine;
using UnityEngine.AddressableAssets;
using UnityEngine.EventSystems;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class ManaDisplay : MonoBehaviour, IPointerClickHandler, IPointerDownHandler, IPointerUpHandler
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] PlayerName _owner;
    [SerializeField] AssetReferenceGameObject _onPress = null!;
    [SerializeField] RectTransform _effectPosition = null!;
    [SerializeField] AssetReferenceGameObject _onChange = null!;
    [SerializeField] TextMeshProUGUI _manaText = null!;
    [SerializeField] TextMeshProUGUI _bonusManaText = null!;
    [SerializeField] TextMeshProUGUI _manaSymbol = null!;
    [SerializeField] uint _currentMana = 5;
    [SerializeField] uint _currentBonusMana;
    
    bool _animationDisabled;
    bool _canTakeGainManaAction;

    public uint CurrentMana => _currentMana;

    bool Clickable => _owner == PlayerName.User &&
                      _canTakeGainManaAction &&
                      _registry.CapabilityService.CanExecuteAction(ClientAction.ActionOneofCase.GainMana);

    public void RenderManaDisplay(ManaView manaView)
    {
      SetMana(manaView.BaseMana);
      SetBonusMana(manaView.BonusMana);
      _canTakeGainManaAction = manaView.CanTakeGainManaAction;
    }

    public void DisableAnimation()
    {
      _manaSymbol.fontMaterial = new Material(Shader.Find("TextMeshPro/Distance Field"));
      _manaSymbol.color = new Color(0.15f, 0.78f, 0.85f);
      _animationDisabled = true;
    }

    public void GainMana(uint amount)
    {
      SetMana(_currentMana + amount);
    }

    public void SpendMana(uint amount)
    {
      Errors.CheckArgument(amount <= _currentMana, "Not enough mana available");
      SetMana(_currentMana - amount);
    }

    void SetMana(uint currentMana)
    {
      Errors.CheckNonNegative(currentMana);

      if (currentMana != _currentMana && !_animationDisabled)
      {
        PlayEffect(_onChange);
      }

      _currentMana = currentMana;
      _manaText.text = "" + _currentMana;
    }

    void SetBonusMana(uint bonusMana)
    {
      Errors.CheckNonNegative(bonusMana);

      if (bonusMana != _currentBonusMana)
      {
        PlayEffect(_onChange);
      }

      _currentBonusMana = bonusMana;

      _bonusManaText.gameObject.SetActive(bonusMana > 0);
      _bonusManaText.text = "" + _currentBonusMana;
    }

    public void OnPointerClick(PointerEventData eventData)
    {
      if (Clickable)
      {
        PlayEffect(_onPress);
        _registry.ActionService.HandleAction(new ClientAction
        {
          GainMana = new GainManaAction()
        });
      }
    }

    void PlayEffect(AssetReferenceGameObject effect)
    {
      StartCoroutine(
        _registry.AssetPoolService.CreateFromReference(effect, _registry.MainCamera.ScreenToWorldPoint(
          new Vector3(_effectPosition.position.x, _effectPosition.position.y, 8f))));      
    }

    public void OnPointerDown(PointerEventData eventData)
    {
      if (Clickable)
      {
        transform.localScale = 0.95f * Vector3.one;
      }
    }

    public void OnPointerUp(PointerEventData eventData)
    {
      if (Clickable)
      {
        transform.localScale = Vector3.one;
      }
    }
  }
}