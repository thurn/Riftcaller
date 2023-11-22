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

using System.Collections.Generic;
using System.Linq;
using Spelldawn.Protos;
using Spelldawn.Services;
using Spelldawn.Utils;
using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class Deck : StackObjectDisplay
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] PlayerName _owner;
    bool _canTakeDrawCardAction;

    bool Clickable => _owner == PlayerName.User &&
                      _canTakeDrawCardAction &&
                      _registry.CapabilityService.CanExecuteAction(ClientAction.ActionOneofCase.DrawCard);    
    
    protected override Registry Registry => _registry;

    protected override GameContext DefaultGameContext() => GameContext.Deck;

    public void RenderDeckView(DeckView deckView)
    {
      _canTakeDrawCardAction = deckView.CanTakeDrawCardAction;
      
      foreach (var obj in AllObjects)
      {
        var spriteRenderer = obj.GetComponent<SpriteRenderer>();
        if (spriteRenderer)
        {
          // Update card back images for placeholder cards
          _registry.AssetService.AssignSprite(spriteRenderer, deckView.CardBack);
        }
      }
    }

    public IEnumerable<Card> Cards() => AllObjects.OfType<Card>();

    protected override void LongPress()
    {
      StartCoroutine(_registry.LongPressCardBrowser.BrowseCards(this));
    }
    
    public override bool CanHandleMouseEvents() => true;
    
    public override void MouseUp()
    {
      if (Clickable)
      {
        if (_registry.CapabilityService.CanExecuteAction(ClientAction.ActionOneofCase.DrawCard))
        {
          _registry.ActionService.HandleAction(new ClientAction
          {
            DrawCard = new DrawCardAction()
          });
        }
        else
        {
          LogUtils.Log("Ignoring click on deck, cannot currently draw a card");
        }
      }
    }
  }
}