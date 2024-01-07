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
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using DG.Tweening;
using Riftcaller.Game;
using Riftcaller.Masonry;
using Riftcaller.Protos;
using Riftcaller.Utils;
using UnityEngine;

#nullable enable

namespace Riftcaller.Services
{
  public sealed class CardService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] BoxCollider _playCardArea = null!;
    [SerializeField] Card _cardPrefab = null!;
    [SerializeField] Card _tokenCardPrefab = null!;
    [SerializeField] Card _fullHeightCardPrefab = null!;
    [SerializeField] Card _fullHeightCardTokenPrefab = null!;

    SpriteAddress _userCardBack = null!;
    SpriteAddress _opponentCardBack = null!;
    GameObject? _currentTargetHighlight;
    bool _showingInfoZoom;

    readonly RaycastHit[] _raycastHitsTempBuffer = new RaycastHit[8];
    readonly Dictionary<CardIdentifier, Card> _cards = new();

    public bool CurrentlyDragging { get; set; }

    public IEnumerator Sync(List<CardView> views, GameObjectPositions? positions, bool animate = true,
      bool delete = true)
    {
      var toDelete = _cards.Keys.ToHashSet();
      var coroutines = new List<Coroutine>();
      
      // Sort the views by position type so that 'stacked behind card' comes after its parent.
      views.Sort((x, y) => x.CardPosition.PositionCase.CompareTo(y.CardPosition.PositionCase));

      foreach (var view in views)
      {
        toDelete.Remove(view.CardId);
        Card card;

        if (_cards.ContainsKey(view.CardId))
        {
          card = _cards[view.CardId];
        }
        else
        {
          card = InstantiateCardPrefab(view.Prefab);
          card.gameObject.SetActive(false);
          _cards[view.CardId] = card;
          card.transform.localScale = new Vector3(Card.CardScale, Card.CardScale, 1f);
          var position = view.CreatePosition ?? Errors.CheckNotNull(view.CardPosition);
          _registry.ObjectPositionService.MoveGameObjectImmediate(card, position);
        }

        card.Render(view, animate: animate);
        
        // Need to update all cards in case sorting keys change
        coroutines.Add(StartCoroutine(
          _registry.ObjectPositionService.MoveGameObject(card, view.CardPosition, animate)));
        
        card.gameObject.SetActive(true);
      }

      if (positions != null)
      {
        coroutines.Add(StartCoroutine(_registry.ObjectPositionService.MoveByIdentifier(
          IdUtil.DeckObjectId(PlayerName.User), positions.UserDeck, animate)));
        coroutines.Add(StartCoroutine(_registry.ObjectPositionService.MoveByIdentifier(
          IdUtil.DeckObjectId(PlayerName.Opponent), positions.OpponentDeck, animate)));
        coroutines.Add(StartCoroutine(_registry.ObjectPositionService.MoveByIdentifier(
          IdUtil.CharacterObjectId(PlayerName.User), positions.UserCharacter, animate)));
        coroutines.Add(StartCoroutine(_registry.ObjectPositionService.MoveByIdentifier(
          IdUtil.CharacterObjectId(PlayerName.Opponent), positions.OpponentCharacter, animate)));
        coroutines.Add(StartCoroutine(_registry.ObjectPositionService.MoveByIdentifier(
          IdUtil.DiscardPileObjectId(PlayerName.User), positions.UserDiscard, animate)));
        coroutines.Add(StartCoroutine(_registry.ObjectPositionService.MoveByIdentifier(
          IdUtil.DiscardPileObjectId(PlayerName.Opponent), positions.OpponentDiscard, animate)));
      }

      if (delete)
      {
        coroutines.AddRange(
          toDelete.Select(cardId => StartCoroutine(HandleDestroyCard(cardId, animate))));
      }

      foreach (var coroutine in coroutines)
      {
        yield return coroutine;
      }

      if (positions != null)
      {
        _registry.GameCharacterForPlayer(PlayerName.User).SetFacingDirection(positions.UserCharacterFacing);
        _registry.GameCharacterForPlayer(PlayerName.Opponent).SetFacingDirection(positions.OpponentCharacterFacing);
      }

      // Wait for the browser which the optimistic card gets added to
      yield return _registry.RevealedCardsBrowserSmall.WaitUntilIdle();
    }

    public void SetDeckViews(DeckView? userDeckView, DeckView? opponentDeckView)
    {
      if (userDeckView != null)
      {
        _registry.DeckForPlayer(PlayerName.User).RenderDeckView(userDeckView);
        _userCardBack = userDeckView.CardBack;
      }

      if (opponentDeckView != null)
      {
        _registry.DeckForPlayer(PlayerName.Opponent).RenderDeckView(opponentDeckView);
        _opponentCardBack = opponentDeckView.CardBack;
      }
    }

    public SpriteAddress GetCardBack(PlayerName playerName) =>
      Errors.CheckNotDefault(playerName) == PlayerName.User ? _userCardBack : _opponentCardBack;

    public bool HasCard(CardIdentifier cardId) => _cards.ContainsKey(cardId);

    public Card FindCard(CardIdentifier cardId)
    {
      Errors.CheckState(_cards.ContainsKey(cardId), $"Card Id {cardId} not found");
      return _cards[cardId];
    }

    public Card CreateCard(CardView cardView, GameContext gameContext, bool animate)
    {
      var card = InstantiateCardPrefab(cardView.Prefab);
      card.transform.localScale = new Vector3(Card.CardScale, Card.CardScale, 1f);
      card.Render(cardView, gameContext, animate: animate);
      return card;
    }

    public IEnumerator HandleTurnFaceDownArenaAnimation(TurnFaceDownArenaAnimationCommand command)
    {
      var card = FindCard(command.CardId);
      return card.PlayFlipAnimation();
    }
    
    public bool IsMouseOverPlayCardArea()
    {
      var ray = _registry.MainCamera.ScreenPointToRay(Input.mousePosition);
      var hits = Physics.RaycastNonAlloc(ray, _raycastHitsTempBuffer, 100);
      for (var i = 0; i < hits; ++i)
      {
        var hit = _raycastHitsTempBuffer[i];
        if (hit.collider == _playCardArea)
        {
          return true;
        }
      }

      return false;
    }

    public void HandleInfoZoomCommand(InfoZoomCommand command)
    {
      if (command is { Show: true, Card: {} view })
      {
        var card = CreateCard(view, GameContext.InfoZoom, animate: false);
        InfoZoom(card);
      }
      else
      {
        ClearInfoZoom();
      }
    }

    public void OnCommandsFinished()
    {
      foreach (var card in _cards.Values)
      {
        card.ClearMovementEffect();
      }
    }

    public void HandleSetCardMovementEffect(SetCardMovementEffectCommand command)
    {
      var card = FindCard(command.CardId);
      var projectile = _registry.AssetPoolService.Create(
        _registry.AssetService.GetProjectile(command.Projectile), card.transform.position);      
      card.SetMovementEffect(projectile.gameObject);
    }
    
    public void DisplayInfoZoom(Card card)
    {
      if (!_showingInfoZoom)
      {
        _showingInfoZoom = true;
        InfoZoom(card);        
      }
    }

    public void ClearInfoZoom()
    {
      _registry.ArenaService.HideRoomSelector();
      _registry.DocumentService.ClearInfoZoom();
      _registry.Studio.ClearSubject();
      if (_currentTargetHighlight)
      {
        Destroy(_currentTargetHighlight);
      }

      _showingInfoZoom = false;
    }

    void InfoZoom(Card card)
    {
      // Don't make this a coroutine, everything will be terrible
      
      ClearInfoZoom();

      var showOnLeft = Input.mousePosition.x > Screen.width / 2.0;
      var zoomed = InfoCopy(card);

      if (zoomed.InfoZoomHighlight != null)
      {
        switch (zoomed.InfoZoomHighlight.HighlightCase)
        {
          case InfoZoomHighlight.HighlightOneofCase.Card:
            showOnLeft = ShowCardHighlight(zoomed.InfoZoomHighlight.Card);
            break;
          case InfoZoomHighlight.HighlightOneofCase.Room:
            showOnLeft = _registry.ArenaService.ShowRoomSelectorForRoom(zoomed.InfoZoomHighlight.Room);
            break;
        }
      }
      
      if (zoomed.SupplementalInfo != null)
      {
        zoomed.SupplementalInfo.Style.Margin = MasonUtil.GroupDip(32f, -120f, 0f, -120f);
        zoomed.SupplementalInfo.Style.AlignItems = showOnLeft ? FlexAlign.FlexStart : FlexAlign.FlexEnd;
      }
      
      var node = MasonUtil.Row("InfoZoom",
        new FlexStyle
        {
          Position = FlexPosition.Absolute,
          Inset = new DimensionGroup
          {
            Left = showOnLeft ? MasonUtil.Px(0) : null,
            Right = showOnLeft ? null : MasonUtil.Px(0)
          }
        },
        showOnLeft ? null : zoomed.SupplementalInfo,
        MasonUtil.Column("Image",
          new FlexStyle
          {
            Width = MasonUtil.Px(625),
            Height = MasonUtil.Px(625),
            BackgroundImage = new NodeBackground
            {
              RenderTexture = new RenderTextureAddress
              {
                Address = Studio.TextureAddress
              }
            }
          }),
        showOnLeft ? zoomed.SupplementalInfo : null
      );

      // Always do this second because LoadAssetsForNode takes 1 frame minimum
      _registry.Studio.SetSubject(zoomed.gameObject);
      
      _registry.DocumentService.RenderInfoZoom(node);
      
      // HACK: The Card component gets disabled during this process one time in a hundred and I have no idea why  
      zoomed.enabled = true;
    }
    
    bool ShowCardHighlight(CardIdentifier cardIdentifier)
    {
      var target = FindCard(cardIdentifier);
      _currentTargetHighlight = new GameObject("Card Info Zoom Highlight");
      var spriteRenderer = _currentTargetHighlight.AddComponent<SpriteRenderer>();
      spriteRenderer.sprite = _registry.StaticAssets.Selector;
      spriteRenderer.transform.eulerAngles = new Vector3(90, 0, 0);
      spriteRenderer.transform.position = target.transform.position;
      return _registry.MainCamera.WorldToScreenPoint(target.transform.position).x > Screen.width / 2.0;
    }

    /// <summary>
    /// Creates a clone of a card for display at large size
    /// </summary>
    Card InfoCopy(Card card)
    {
      var zoomed = card.CloneForDisplay();
      zoomed.Parent = null;
      zoomed.transform.position = Vector3.zero;
      zoomed.transform.localScale = new Vector3(Card.CardScale, Card.CardScale, 1f);
      zoomed.SetGameContext(GameContext.InfoZoom);
      zoomed.gameObject.name = $"{card.name} Info";
      return zoomed;
    }
    
    public IEnumerator HandleDestroyCard(CardIdentifier cardId, bool animate)
    {
      var card = FindCard(cardId);

      Sequence? sequence = null;

      if (card.DestroyPosition != null)
      {
        if (card.DestroyPosition.PositionCase == ObjectPosition.PositionOneofCase.Deck)
        {
          sequence = card.TurnFaceDown(animate);          
        }
        yield return _registry.ObjectPositionService.MoveGameObject(card, card.DestroyPosition, animate);
      }

      _cards.Remove(cardId);

      if (card.Parent != null && card.Parent.AsMonoBehaviour())
      {
        var parent = card.Parent;
        parent!.RemoveObjectIfPresent(card, animate: false);
      }

      if (sequence != null && sequence.IsActive() && !sequence.IsComplete())
      {
        yield return sequence.WaitForCompletion();
      }

      Destroy(card.gameObject);
    }

    Card InstantiateCardPrefab(CardPrefab prefab)
    {
      var result = ComponentUtils.Instantiate(prefab switch
      {
        CardPrefab.Standard => _cardPrefab,
        CardPrefab.TokenCard => _tokenCardPrefab,
        CardPrefab.FullHeight => _fullHeightCardPrefab,
        CardPrefab.FullHeightToken => _fullHeightCardTokenPrefab,
        _ => throw new ArgumentOutOfRangeException()
      });
      result.Registry = _registry;
      return result;
    }
  }
}