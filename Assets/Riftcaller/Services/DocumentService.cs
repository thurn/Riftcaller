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
using System.Linq;
using Riftcaller.Common;
using Riftcaller.Game;
using Riftcaller.Masonry;
using static Riftcaller.Masonry.MasonUtil;
using Riftcaller.Protos;
using Riftcaller.Utils;
using UnityEngine;
using UnityEngine.Serialization;
using UnityEngine.UIElements;

#nullable enable

namespace Riftcaller.Services
{
  public sealed record ElementPosition
  {
    public float Top { get; init; }
    public float Right { get; init; }
    public float Bottom { get; init; }
    public float Left { get; init; }
  }

  public enum ScreenMode
  {
    ConstantPhysicalSize,
    ReferenceWidth1920
  }  
  
  public sealed class DocumentService : MonoBehaviour
  {
    public const ScreenMode DefaultScreenMode = ScreenMode.ReferenceWidth1920;
    
    [SerializeField] Registry _registry = null!;
    [SerializeField] UIDocument _document = null!;
    [FormerlySerializedAs("_loadingIndicator")] [SerializeField] Sprite _loadingSprite = null!;

    readonly List<InterfacePanelAddress> _openPanels = new();
    readonly Dictionary<InterfacePanelAddress, InterfacePanel> _panelCache = new();
    readonly HashSet<InterfacePanelAddress> _waitingFor = new();
    InterfacePanelAddress? _switchTo;
    VisualElement _mainOverlay = null!;
    VisualElement _mainControls = null!;
    VisualElement _cardControls = null!;
    VisualElement _infoZoom = null!;
    VisualElement _panels = null!;
    BottomSheet _bottomSheet = null!;
    LoadingSpinner? _loadingSpinner;
    VisualElement _screenOverlay = null!;
    Node? _screenOverlayNode;
    Coroutine? _autoRefresh;

    public VisualElement RootVisualElement => _document.rootVisualElement;

    public IEnumerable<InterfacePanelAddress> OpenPanels => _openPanels;

    public IReadOnlyDictionary<InterfacePanelAddress, InterfacePanel> PanelCache => _panelCache;

    public void Initialize()
    {
      _document.rootVisualElement.Clear();
      AddRoot("Main Overlay", out _mainOverlay);
      AddRoot("Main Controls", out _mainControls);
      AddRoot("Card Controls", out _cardControls);
      AddRoot("Panels", out _panels);
      _bottomSheet = new BottomSheet(_registry);
      _document.rootVisualElement.Add(_bottomSheet);
      
      AddRoot("Loading", out var loadingContainer);
      _loadingSpinner = new LoadingSpinner();
      _loadingSpinner.Initialize(loadingContainer, _loadingSprite);
      
      AddRoot("ScreenOverlay", out _screenOverlay);
      AddRoot("InfoZoom", out _infoZoom);
    }

    void Update()
    {
      _loadingSpinner?.Update();
    }

    public void WaitFor(WaitingFor waitingFor) => _loadingSpinner?.WaitFor(waitingFor);
    
    public void EndWaitFor(WaitingFor waitingFor) => _loadingSpinner?.EndWaitFor(waitingFor);    
    
    public void FetchOpenPanelsOnConnect()
    {
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

    public float ScreenPxToElementPx(ScreenMode mode, float value) => mode switch
    {
      ScreenMode.ConstantPhysicalSize => value * _document.panelSettings.referenceDpi / Screen.dpi,
      ScreenMode.ReferenceWidth1920 => value * (1920f / Screen.width),
      _ => throw new ArgumentOutOfRangeException(nameof(mode), mode, null)
    };

    /// <summary>
    /// Returns an ElementPosition in interface coordinates corresponding to a screen position.
    /// </summary>
    public ElementPosition ScreenPositionToElementPosition(ScreenMode mode, Vector3 screenPosition) =>
      new()
      {
        Top = ScreenPxToElementPx(mode, Screen.height - screenPosition.y),
        Right = ScreenPxToElementPx(mode, Screen.width - screenPosition.x),
        Bottom = ScreenPxToElementPx(mode, screenPosition.y),
        Left = ScreenPxToElementPx(mode, screenPosition.x)
      };

    public Vector2 ElementMousePosition()
    {
      var position = ScreenPositionToElementPosition(DefaultScreenMode, Input.mousePosition);
      return new Vector2(position.Left, position.Top);
    }

    public bool MouseOverScreenElement()
    {
      return _panels.Children()
        .Concat(_screenOverlay.Children())
        .Concat(_mainControls.Children())
        .Concat(_mainOverlay.Children())
        .Any(c => c.ContainsPoint(c.WorldToLocal(ElementMousePosition())));
    }

    /// <summary>
    /// Returns an ElementPosition in interface coordinates corresponding to the position of the
    /// provided transform.
    /// </summary>
    public ElementPosition TransformPositionToElementPosition(Transform t)
      => ScreenPositionToElementPosition(DefaultScreenMode, _registry.MainCamera.WorldToScreenPoint(t.position));

    public void TogglePanel(TogglePanelCommand command)
    {
      InterfacePanelAddress? fetch = null;
      switch (command.ToggleCommandCase)
      {
        case TogglePanelCommand.ToggleCommandOneofCase.Transition:
          var transition = command.Transition;
          if (transition.Open != null)
          {
            if (!_openPanels.Contains(transition.Open))
            {
              _openPanels.Add(transition.Open);
            }
            
            if (!_panelCache.ContainsKey(transition.Open))
            {
              if (transition.Loading != null && _panelCache.ContainsKey(transition.Loading))
              {
                _panelCache[transition.Open] = new InterfacePanel { Node = _panelCache[transition.Loading].Node };
                _waitingFor.Add(transition.Open);
                WaitFor(WaitingFor.PanelFetch);
                fetch = transition.Open;
              }
              else if (transition.WaitToLoad)
              {
                if (transition.Close != null && _panelCache.ContainsKey(transition.Close))
                {
                  _panelCache[transition.Open] = _panelCache[transition.Close];
                }
                
                _waitingFor.Add(transition.Open);
                WaitFor(WaitingFor.PanelFetch);
                fetch = transition.Open;
              }
              else
              {
                throw new InvalidOperationException($"Attempted to open {transition.Open} with no loading state");
              }
            }
          }

          if (transition.Close != null)
          {
            _openPanels.Remove(transition.Close);
          }

          RenderPanels();
          break;
        case TogglePanelCommand.ToggleCommandOneofCase.OpenBottomSheetAddress:
          fetch = command.OpenBottomSheetAddress;
          StartCoroutine(_bottomSheet.OpenWithAddress(command.OpenBottomSheetAddress));
          break;
        case TogglePanelCommand.ToggleCommandOneofCase.CloseBottomSheet:
          StartCoroutine(_bottomSheet.Close());
          break;
        case TogglePanelCommand.ToggleCommandOneofCase.PushBottomSheetAddress:
          fetch = command.PushBottomSheetAddress;
          StartCoroutine(_bottomSheet.PushAddress(command.PushBottomSheetAddress));
          break;
        case TogglePanelCommand.ToggleCommandOneofCase.PopToBottomSheetAddress:
          fetch = command.PopToBottomSheetAddress;
          StartCoroutine(_bottomSheet.PopToAddress(command.PopToBottomSheetAddress));
          break;
        default:
          throw new ArgumentOutOfRangeException();
      }

      if (fetch != null)
      {
        _registry.ActionService.HandleAction(new ClientAction
        {
          FetchPanel = new FetchPanelAction
          {
            PanelAddress = fetch
          }
        });
      }
    }

    public bool IsOpen(InterfacePanelAddress address) => _openPanels.Contains(address);

    public bool IsAnyPanelOpen() => _openPanels.Count > 0;

    public void HandleUpdatePanels(UpdatePanelsCommand command)
    {
      foreach (var panel in command.Panels)
      {
        _waitingFor.Remove(panel.Address);

        if (_switchTo != null && _switchTo.Equals(panel.Address))
        {
          _openPanels.Clear();
          _openPanels.Add(_switchTo);
          _switchTo = null;
          EndWaitFor(WaitingFor.PanelFetch);
        }

        _panelCache[panel.Address] = panel;
      }

      if (_waitingFor.Count == 0)
      {
        EndWaitFor(WaitingFor.PanelFetch);
      }

      RenderPanels();
    }

    public void RenderMainControls(InterfaceMainControls? mainControls)
    {
      Reconcile(ref _mainOverlay, MainOverlay(mainControls?.Overlay));
      
      Reconcile(ref _mainControls, MainControls(mainControls?.Node));

      Reconcile(
        ref _cardControls,
        CardAnchors(mainControls?.CardAnchorNodes ?? Enumerable.Empty<CardAnchorNode>()));
    }
    
    public void ClearMainControls()
    {
      Reconcile(ref _mainControls, MainControls(null));
      Reconcile(ref _cardControls, CardAnchors(Enumerable.Empty<CardAnchorNode>()));
    }    

    public void AddRequestFields(StandardAction action)
    {
      foreach (var key in action.RequestFields.Keys)
      {
        var element = RootVisualElement.Q<BaseField<string>>(key);
        action.RequestFields[key] = element.value;
      }
    }

    public void SetScreenOverlay(Node? screenOverlay)
    {
      _screenOverlayNode = screenOverlay;
      UpdateScreenOverlay();
    }

    void RenderPanels()
    {
      Reconcile(
        ref _panels,
        Panels(_openPanels.Select(p => _panelCache.GetValueOrDefault(p)?.Node).WhereNotNull()));

      _bottomSheet.RefreshPanels();
      UpdateScreenOverlay();
    }

    void UpdateScreenOverlay()
    {
      var overlay = _openPanels
        .Select(p => _panelCache.GetValueOrDefault(p)?.ScreenOverlay).WhereNotNull()
        .LastOrDefault();      
      Reconcile(ref _screenOverlay, overlay ?? _screenOverlayNode ?? new Node());      
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

    public DimensionGroup GetSafeArea()
    {
      var panel = RootVisualElement.panel;
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

    Node Panels(IEnumerable<Node> children) =>
      Row("Panels", new FlexStyle
      {
        Position = FlexPosition.Absolute,
        Inset = AllDip(0),
      }, children);
    
    static Node MainOverlay(Node? content) =>
      Row("MainOverlay", new FlexStyle
      {
        Position = FlexPosition.Absolute,
        Inset = AllDip(0),
      }, content);    

    static Node MainControls(Node? content) =>
      Row("MainControls", new FlexStyle
      {
        Position = FlexPosition.Absolute,
        Height = Px(125),
        JustifyContent = FlexJustify.FlexEnd,
        Inset = new DimensionGroup
        {
          Left = Px(0),
          Right = Px(0),
          Bottom = Px(220)
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