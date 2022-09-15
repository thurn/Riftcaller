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

using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using Spelldawn.Game;
using Spelldawn.Masonry;
using static Spelldawn.Masonry.MasonUtil;
using Spelldawn.Protos;
using Spelldawn.Tools;
using Spelldawn.Utils;
using UnityEngine;
using UnityEngine.UIElements;

#nullable enable

namespace Spelldawn.Services
{
  public sealed record ElementPosition
  {
    public float Top { get; init; }
    public float Right { get; init; }
    public float Bottom { get; init; }
    public float Left { get; init; }
  }

  public sealed class DocumentService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] UIDocument _document = null!;
    readonly List<InterfacePanelAddress> _openPanels = new();
    readonly Dictionary<InterfacePanelAddress, Node> _panelCache = new();

    VisualElement _fullScreen = null!;
    VisualElement _mainControls = null!;
    VisualElement _cardControls = null!;
    VisualElement _infoZoom = null!;
    VisualElement? _currentlyDragging;
    Coroutine? _autoRefresh;

    public VisualElement RootVisualElement => _document.rootVisualElement;

    public IEnumerable<InterfacePanelAddress> OpenPanels => _openPanels;

    public void Initialize()
    {
      _document.rootVisualElement.Clear();
      AddRoot("Main Controls", out _mainControls);
      AddRoot("Card Controls", out _cardControls);
      AddRoot("InfoZoom", out _infoZoom);
      AddRoot("Full Screen", out _fullScreen);
    }

    void Update()
    {
      if (_autoRefresh == null && AutoRefreshPreference.AutomaticallyRefreshPanels)
      {
        _autoRefresh = StartCoroutine(AutoRefresh());
      }
      else if (_autoRefresh != null && !AutoRefreshPreference.AutomaticallyRefreshPanels)
      {
        StopCoroutine(_autoRefresh);
        _autoRefresh = null;
      }
    }

    IEnumerator AutoRefresh()
    {
      while (true)
      {
        yield return new WaitForSeconds(1.0f);

        foreach (var address in _openPanels)
        {
          _registry.ActionService.HandleAction(new ClientAction
          {
            FetchPanel = new FetchPanelAction
            {
              PanelAddress = address
            }
          });
        }
      }
      // ReSharper disable once IteratorNeverReturns
    }

    public float ScreenPxToElementPx(float value) => value * _document.panelSettings.referenceDpi / Screen.dpi;

    /// <summary>
    /// Returns an ElementPosition in interface coordinates corresponding to a screen position.
    /// </summary>
    public ElementPosition ScreenPositionToElementPosition(Vector3 screenPosition) =>
      new()
      {
        Top = ScreenPxToElementPx(Screen.height - screenPosition.y),
        Right = ScreenPxToElementPx(Screen.width - screenPosition.x),
        Bottom = ScreenPxToElementPx(screenPosition.y),
        Left = ScreenPxToElementPx(screenPosition.x)
      };

    public Vector2 ElementMousePosition()
    {
      var position = ScreenPositionToElementPosition(Input.mousePosition);
      return new Vector2(position.Left, position.Top);
    }

    public bool MouseOverFullScreenElement()
    {
      return _fullScreen.Children().Any(c => c.ContainsPoint(c.WorldToLocal(ElementMousePosition())));
    }

    /// <summary>
    /// Returns an ElementPosition in interface coordinates corresponding to the position of the
    /// provided transform.
    /// </summary>
    public ElementPosition TransformPositionToElementPosition(Transform t)
      => ScreenPositionToElementPosition(_registry.MainCamera.WorldToScreenPoint(t.position));

    public void TogglePanel(bool open, InterfacePanelAddress address)
    {
      if (open)
      {
        if (!_openPanels.Contains(address))
        {
          _openPanels.Add(address);
        }

        _registry.ActionService.HandleAction(new ClientAction
        {
          FetchPanel = new FetchPanelAction
          {
            PanelAddress = address
          }
        });
      }
      else
      {
        _openPanels.Remove(address);
      }

      RenderPanels();
    }

    public bool IsOpen(InterfacePanelAddress address) => _openPanels.Contains(address);

    public bool IsAnyPanelOpen() => _openPanels.Count > 0;

    public void HandleUpdatePanels(UpdatePanelsCommand command)
    {
      foreach (var panel in command.Panels)
      {
        _panelCache[panel.Address] = panel.Node;
      }

      RenderPanels();
    }

    public void RenderMainControls(InterfaceMainControls? mainControls)
    {
      Reconcile(
        ref _mainControls,
        MainControls(mainControls?.Node));

      Reconcile(
        ref _cardControls,
        CardAnchors(mainControls?.CardAnchorNodes ?? Enumerable.Empty<CardAnchorNode>()));
    }

    void RenderPanels()
    {
      Reconcile(
        ref _fullScreen,
        FullScreen(_openPanels.Select(p => _panelCache.GetValueOrDefault(p)).WhereNotNull()));
    }

    void Reconcile(ref VisualElement previousElement, Node newNode)
    {
      var result = Reconciler.Update(_registry, newNode, previousElement);

      if (result != null)
      {
        previousElement = result;
      }
    }

    public void ClearInfoZoom()
    {
      Reconcile(ref _infoZoom, new Node());
    }

    public void RenderInfoZoom(Node node)
    {
      Reconcile(ref _infoZoom, node);
    }

    void AddRoot(string elementName, out VisualElement element)
    {
      var node = Row(elementName, new FlexStyle
      {
        Position = FlexPosition.Absolute,
        Inset = AllDip(0),
        PickingMode = FlexPickingMode.Ignore
      });
      element = Mason.Render(_registry, node);
      _document.rootVisualElement.Add(element);
    }

    DimensionGroup GetSafeArea(IPanel panel)
    {
      var safeLeftTop = RuntimePanelUtils.ScreenToPanel(
        panel,
        new Vector2(Screen.safeArea.xMin, Screen.height - Screen.safeArea.yMax)
      );
      var safeRightBottom = RuntimePanelUtils.ScreenToPanel(
        panel,
        new Vector2(Screen.width - Screen.safeArea.xMax, Screen.safeArea.yMin)
      );

      return GroupDip(top: safeLeftTop.y, right: safeRightBottom.x, bottom: safeRightBottom.y,
        left: safeLeftTop.x);
    }
    
    Node FullScreen(IEnumerable<Node> children) =>
      Row("FullScreen", new FlexStyle
      {
        Position = FlexPosition.Absolute,
        Padding = GetSafeArea(RootVisualElement.panel),
        Inset = AllDip(0),
      }, children);

    static Node MainControls(Node? content) =>
      Row("MainControls", new FlexStyle
      {
        Position = FlexPosition.Absolute,
        Height = Px(125),
        Inset = new DimensionGroup
        {
          Left = Px(0),
          Right = Px(0),
          Bottom = Px(160)
        }
      }, content);

    Node CardAnchors(IEnumerable<CardAnchorNode> nodes)
    {
      return Row("CardAnchors", new FlexStyle
      {
        Position = FlexPosition.Absolute,
        Inset = AllDip(0),
      }, nodes.Select(RenderCardAnchorNode));
    }

    Node RenderCardAnchorNode(CardAnchorNode anchorNode) =>
      AnchorToCard(_registry.CardService.FindCard(anchorNode.CardId), anchorNode.Node, anchorNode.Anchors);

    Node AnchorToCard(Card card, Node node, IEnumerable<CardAnchor> anchors)
    {
      node.Style.Position = FlexPosition.Absolute;
      var inset = new DimensionGroup();

      foreach (var anchor in anchors)
      {
        var target = anchor.CardCorner switch
        {
          AnchorCorner.TopLeft => card.TopLeftAnchor,
          AnchorCorner.TopRight => card.TopRightAnchor,
          AnchorCorner.BottomLeft => card.BottomLeftAnchor,
          AnchorCorner.BottomRight => card.BottomRightAnchor,
          _ => throw new ArgumentOutOfRangeException()
        };

        var position = TransformPositionToElementPosition(target);

        switch (anchor.NodeCorner)
        {
          case AnchorCorner.TopLeft:
            inset.Left = Px(position.Left);
            inset.Top = Px(position.Top);
            break;
          case AnchorCorner.TopRight:
            inset.Right = Px(position.Right);
            inset.Top = Px(position.Top);
            break;
          case AnchorCorner.BottomLeft:
            inset.Left = Px(position.Left);
            inset.Bottom = Px(position.Bottom);
            break;
          case AnchorCorner.BottomRight:
            inset.Right = Px(position.Right);
            inset.Bottom = Px(position.Bottom);
            break;
          default:
            throw new ArgumentOutOfRangeException();
        }
      }

      node.Style.Inset = inset;
      return node;
    }
  }
}