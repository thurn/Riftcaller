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
using System.Collections.Generic;
using System.Linq;
using Spelldawn.Protos;
using Spelldawn.Services;
using Spelldawn.Utils;
using UnityEngine;
using UnityEngine.UIElements;
using EasingMode = Spelldawn.Protos.EasingMode;
using FlexDirection = Spelldawn.Protos.FlexDirection;
using FontStyle = Spelldawn.Protos.FontStyle;
using OverflowClipBox = Spelldawn.Protos.OverflowClipBox;
using TextOverflow = Spelldawn.Protos.TextOverflow;
using TextOverflowPosition = Spelldawn.Protos.TextOverflowPosition;
using TextShadow = UnityEngine.UIElements.TextShadow;
using TimeValue = UnityEngine.UIElements.TimeValue;
using WhiteSpace = Spelldawn.Protos.WhiteSpace;

#nullable enable

namespace Spelldawn.Masonry
{
  public static class Mason
  {
    /// <summary>
    /// Renders the provided Node into a VisualElement, recursively rendering child nodes.
    /// </summary>
    public static VisualElement Render(Registry registry, Node node)
    {
      var element = CreateElement(node);
      ApplyToElement(registry, element, node);

      foreach (var child in node.Children)
      {
        element.Add(Render(registry, child));
      }

      return element;
    }

    public static VisualElement CreateElement(Node node)
    {
      VisualElement result = node.NodeType?.NodeTypeCase switch
      {
        NodeType.NodeTypeOneofCase.Text => new NodeLabel(),
        NodeType.NodeTypeOneofCase.ScrollViewNode => new NodeScrollView(),
        NodeType.NodeTypeOneofCase.DraggableNode => new Draggable(),
        NodeType.NodeTypeOneofCase.DropTargetNode => new DropTarget(),
        NodeType.NodeTypeOneofCase.TextFieldNode => new NodeTextField(),
        NodeType.NodeTypeOneofCase.SliderNode => new NodeSlider(),
        _ => new NodeVisualElement()
      };
      ((IMasonElement)result).Node = node;

      return result;
    }

    /// <summary>Applies the configuration in a Node to an existing VisualElement, without modifying children.</summary>
    public static void ApplyToElement(Registry registry, VisualElement element, Node node)
    {
      switch (node.NodeType?.NodeTypeCase)
      {
        case NodeType.NodeTypeOneofCase.Text:
          ApplyText((NodeLabel)element, node.NodeType.Text);
          break;
        case NodeType.NodeTypeOneofCase.ScrollViewNode:
          ScrollViews.Apply(registry, (NodeScrollView)element, node.NodeType.ScrollViewNode);
          break;
        case NodeType.NodeTypeOneofCase.DraggableNode:
          Draggable.Apply(registry, (Draggable)element, node);
          break;
        case NodeType.NodeTypeOneofCase.TextFieldNode:
          TextFields.Apply(registry, (NodeTextField)element, node.NodeType.TextFieldNode);
          break;
        case NodeType.NodeTypeOneofCase.SliderNode:
          Sliders.Apply(registry, (NodeSlider)element, node.NodeType.SliderNode);
          break;
      }

      ApplyNode(registry, node, element);
    }

    static void ApplyNode(Registry registry, Node node, VisualElement element)
    {
      element.name = node.Name;

      ApplyStyle(registry, element, node.Style);

      if (element is INodeCallbacks callbacks)
      {
        if (node.HoverStyle != null)
        {
          var hoverStyle = new FlexStyle();
          hoverStyle.MergeFrom(node.Style);
          hoverStyle.MergeFrom(node.HoverStyle);
          callbacks.SetCallback(Callbacks.Event.MouseEnter, () => { ApplyStyle(registry, element, hoverStyle); });
          callbacks.SetCallback(Callbacks.Event.MouseLeave, () => { ApplyStyle(registry, element, node.Style); });
        }
        else
        {
          callbacks.SetCallback(Callbacks.Event.MouseEnter, null);
          callbacks.SetCallback(Callbacks.Event.MouseLeave, null);
        }

        if (node.PressedStyle != null)
        {
          var pressedStyle = new FlexStyle();
          pressedStyle.MergeFrom(node.Style);
          pressedStyle.MergeFrom(node.PressedStyle);
          callbacks.SetCallback(Callbacks.Event.MouseDown, () =>
          {
            ApplyStyle(registry, element, pressedStyle);
            
            if (node.EventHandlers?.OnMouseDown is { } onMouseDown)
            {
              registry.ActionService.HandleAction(onMouseDown);
            }
          });
          callbacks.SetCallback(Callbacks.Event.MouseUp, () =>
          {
            var style = node.Style;
            if (node.HoverStyle != null)
            {
              style = new FlexStyle();
              style.MergeFrom(node.Style);
              style.MergeFrom(node.HoverStyle);
            }

            ApplyStyle(registry, element, style);
            
            if (node.EventHandlers?.OnMouseUp is { } onMouseUp)
            {
              registry.ActionService.HandleAction(onMouseUp);
            }            
          });
        }
        else
        {
          SetCallback(registry, callbacks, node.EventHandlers?.OnMouseDown, Callbacks.Event.MouseDown);
          SetCallback(registry, callbacks, node.EventHandlers?.OnMouseUp, Callbacks.Event.MouseUp);
        }

        SetCallback(registry, callbacks, node.EventHandlers?.OnClick, Callbacks.Event.Click);
        SetCallback(registry, callbacks, node.EventHandlers?.OnLongPress, Callbacks.Event.LongPress);

        if (node.PressedStyle != null || node.HoverStyle != null || node.EventHandlers != null)
        {
          element.pickingMode = PickingMode.Position;
        }
        else
        {
          // Ignore mouse events on non-interactive elements
          element.pickingMode = PickingMode.Ignore;
        }
      }
      else
      {
        if (node.PressedStyle != null || node.HoverStyle != null || node.EventHandlers != null)
        {
          LogUtils.LogError($"Custom element {element} cannot have interaction");
        }
      }
    }
    
    static void SetCallback(Registry registry, INodeCallbacks element, ClientAction? action, Callbacks.Event eventType)
    {
      if (action != null)
      {
        element.SetCallback(eventType, () =>
        {
          registry.ActionService.HandleAction(action);
        });
      }
      else
      {
        element.SetCallback(eventType, null);
      }
    }

    static void ApplyText(Label label, Text text)
    {
      label.text = text.Label;
    }

    public static Color ToUnityColor(FlexColor color) => new(color.Red, color.Green, color.Blue, color.Alpha);

    static StyleColor AdaptColor(FlexColor? color) =>
      color == null ? new StyleColor(StyleKeyword.Null) : ToUnityColor(color);

    static StyleFloat AdaptFloat(float? input) => input ?? new StyleFloat(StyleKeyword.Null);

    static StyleInt AdaptInt(uint? input) => (int?)input ?? new StyleInt(StyleKeyword.Null);

    public static Vector2 AdaptVector2(FlexVector2? input) => input is { } v ? new Vector2(v.X, v.Y) : Vector2.zero;

    public static Vector3 AdaptVector3(FlexVector3? input) => input is { } v ? new Vector3(v.X, v.Y, v.Z) : Vector2.zero;

    static Length AdaptDimensionNonNull(Registry registry, Dimension dimension) => dimension.Unit switch
    {
      DimensionUnit.Pixels => new Length(dimension.Value),
      DimensionUnit.Percentage => Length.Percent(dimension.Value),
      DimensionUnit.ViewportWidth => new Length(
        registry.DocumentService.ScreenPxToElementPx(
          DocumentService.DefaultScreenMode,
          (dimension.Value / 100) * Screen.safeArea.width)),
      DimensionUnit.ViewportHeight => new Length(
        registry.DocumentService.ScreenPxToElementPx(
          DocumentService.DefaultScreenMode,
          (dimension.Value / 100) * Screen.safeArea.height)),
      DimensionUnit.SafeAreaTop => new Length(registry.DocumentService.GetSafeArea().Top.Value * dimension.Value),
      DimensionUnit.SafeAreaRight => new Length(registry.DocumentService.GetSafeArea().Right.Value * dimension.Value),
      DimensionUnit.SafeAreaBottom => new Length(registry.DocumentService.GetSafeArea().Bottom.Value * dimension.Value),
      DimensionUnit.SafeAreaLeft => new Length(registry.DocumentService.GetSafeArea().Left.Value * dimension.Value),      
      _ => throw new ArgumentOutOfRangeException()
    };

    static StyleLength AdaptDimension(Registry registry, Dimension? dimension) =>
      dimension is { } d ? AdaptDimensionNonNull(registry, d) : new StyleLength(StyleKeyword.Null);

    static StyleEnum<Align> AdaptAlign(FlexAlign input) => input switch
    {
      FlexAlign.Auto => Align.Auto,
      FlexAlign.FlexStart => Align.FlexStart,
      FlexAlign.Center => Align.Center,
      FlexAlign.FlexEnd => Align.FlexEnd,
      FlexAlign.Stretch => Align.Stretch,
      _ => new StyleEnum<Align>(StyleKeyword.Null)
    };

    static StyleList<TResult> AdaptList<TSource, TResult>(IList<TSource> field, Func<TSource, TResult> selector) =>
      field.Count == 0
        ? new StyleList<TResult>(StyleKeyword.Null)
        : new StyleList<TResult>(field.Select(selector).ToList());

    public static void ApplyStyle(Registry registry, VisualElement e, FlexStyle? input)
    {
      if (input == null)
      {
        return;
      }

      foreach (var name in e.GetClasses().ToList().Where(name => name.StartsWith("sd_")))
      {
        e.RemoveFromClassList(name);
      }

      e.style.alignContent = AdaptAlign(input.AlignContent);
      e.style.alignItems = AdaptAlign(input.AlignItems);
      e.style.alignSelf = AdaptAlign(input.AlignSelf);
      e.style.backgroundColor = AdaptColor(input.BackgroundColor);
      e.style.borderTopColor = AdaptColor(input.BorderColor?.Top);
      e.style.borderRightColor = AdaptColor(input.BorderColor?.Right);
      e.style.borderBottomColor = AdaptColor(input.BorderColor?.Bottom);
      e.style.borderLeftColor = AdaptColor(input.BorderColor?.Left);
      e.style.borderTopLeftRadius = AdaptDimension(registry, input.BorderRadius?.TopLeft);
      e.style.borderTopRightRadius = AdaptDimension(registry, input.BorderRadius?.TopRight);
      e.style.borderBottomRightRadius = AdaptDimension(registry, input.BorderRadius?.BottomRight);
      e.style.borderBottomLeftRadius = AdaptDimension(registry, input.BorderRadius?.BottomLeft);
      e.style.borderTopWidth = AdaptFloat(input.BorderWidth?.Top);
      e.style.borderRightWidth = AdaptFloat(input.BorderWidth?.Right);
      e.style.borderBottomWidth = AdaptFloat(input.BorderWidth?.Bottom);
      e.style.borderLeftWidth = AdaptFloat(input.BorderWidth?.Left);
      e.style.top = AdaptDimension(registry, input.Inset?.Top);
      e.style.right = AdaptDimension(registry, input.Inset?.Right);
      e.style.bottom = AdaptDimension(registry, input.Inset?.Bottom);
      e.style.left = AdaptDimension(registry, input.Inset?.Left);
      e.style.color = AdaptColor(input.Color);
      e.style.display = input.Display switch
      {
        FlexDisplayStyle.Flex => DisplayStyle.Flex,
        FlexDisplayStyle.None => DisplayStyle.None,
        _ => new StyleEnum<DisplayStyle>(StyleKeyword.Null)
      };
      e.style.flexBasis = AdaptDimension(registry, input.FlexBasis);
      e.style.flexDirection = input.FlexDirection switch
      {
        FlexDirection.Column => UnityEngine.UIElements.FlexDirection.Column,
        FlexDirection.ColumnReverse => UnityEngine.UIElements.FlexDirection.ColumnReverse,
        FlexDirection.Row => UnityEngine.UIElements.FlexDirection.Row,
        FlexDirection.RowReverse => UnityEngine.UIElements.FlexDirection.RowReverse,
        _ => new StyleEnum<UnityEngine.UIElements.FlexDirection>(StyleKeyword.Null)
      };
      e.style.flexGrow = AdaptFloat(input.FlexGrow);
      e.style.flexShrink = AdaptFloat(input.FlexShrink);
      e.style.flexWrap = input.Wrap switch
      {
        FlexWrap.NoWrap => Wrap.NoWrap,
        FlexWrap.Wrap => Wrap.Wrap,
        FlexWrap.WrapReverse => Wrap.WrapReverse,
        _ => new StyleEnum<Wrap>(StyleKeyword.Null)
      };
      e.style.fontSize = AdaptDimension(registry, input.FontSize);
      e.style.height = AdaptDimension(registry, input.Height);
      e.style.justifyContent = input.JustifyContent switch
      {
        FlexJustify.FlexStart => Justify.FlexStart,
        FlexJustify.Center => Justify.Center,
        FlexJustify.FlexEnd => Justify.FlexEnd,
        FlexJustify.SpaceBetween => Justify.SpaceBetween,
        FlexJustify.SpaceAround => Justify.SpaceAround,
        _ => new StyleEnum<Justify>(StyleKeyword.Null)
      };
      e.style.letterSpacing = AdaptDimension(registry, input.LetterSpacing);
      e.style.marginTop = AdaptDimension(registry, input.Margin?.Top);
      e.style.marginRight = AdaptDimension(registry, input.Margin?.Right);
      e.style.marginBottom = AdaptDimension(registry, input.Margin?.Bottom);
      e.style.marginLeft = AdaptDimension(registry, input.Margin?.Left);
      e.style.maxHeight = AdaptDimension(registry, input.MaxHeight);
      e.style.maxWidth = AdaptDimension(registry, input.MaxWidth);
      e.style.minHeight = AdaptDimension(registry, input.MinHeight);
      e.style.minWidth = AdaptDimension(registry, input.MinWidth);
      e.style.opacity = AdaptFloat(input.Opacity);
      e.style.overflow = input.Overflow switch
      {
        FlexOverflow.Visible => Overflow.Visible,
        FlexOverflow.Hidden => Overflow.Hidden,
        _ => new StyleEnum<Overflow>(StyleKeyword.Null)
      };
      e.style.paddingTop = AdaptDimension(registry, input.Padding?.Top);
      e.style.paddingRight = AdaptDimension(registry, input.Padding?.Right);
      e.style.paddingBottom = AdaptDimension(registry, input.Padding?.Bottom);
      e.style.paddingLeft = AdaptDimension(registry, input.Padding?.Left);
      e.style.position = input.Position switch
      {
        FlexPosition.Relative => Position.Relative,
        FlexPosition.Absolute => Position.Absolute,
        _ => new StyleEnum<Position>(StyleKeyword.Null)
      };
      e.style.rotate = input.Rotate is { } r
        ? new Rotate(Angle.Degrees(r.Degrees))
        : new StyleRotate(StyleKeyword.Null);
      e.style.scale = input.Scale is { } s ? new Scale(AdaptVector3(s.Amount)) : new StyleScale(StyleKeyword.Null);
      e.style.textOverflow = input.TextOverflow switch
      {
        TextOverflow.Clip => UnityEngine.UIElements.TextOverflow.Clip,
        TextOverflow.Ellipsis => UnityEngine.UIElements.TextOverflow.Ellipsis,
        _ => new StyleEnum<UnityEngine.UIElements.TextOverflow>(StyleKeyword.Null)
      };
      e.style.textShadow = input.TextShadow is { } ts
        ? new TextShadow
        {
          offset = AdaptVector2(ts.Offset),
          blurRadius = ts.BlurRadius,
          color = ts.Color == null ? Color.black : ToUnityColor(ts.Color)
        }
        : new StyleTextShadow(StyleKeyword.Null);
      e.style.transformOrigin = input.TransformOrigin is { } to
        ? new TransformOrigin(AdaptDimensionNonNull(registry, to.X), AdaptDimensionNonNull(registry, to.Y), to.Z)
        : new StyleTransformOrigin(StyleKeyword.Null);
      e.style.transitionDelay =
        AdaptList(input.TransitionDelays, t => new TimeValue(t.Milliseconds, TimeUnit.Millisecond));
      e.style.transitionDuration = AdaptList(input.TransitionDurations,
        t => new TimeValue(t.Milliseconds, TimeUnit.Millisecond));
      e.style.transitionProperty = AdaptList(input.TransitionProperties, p => new StylePropertyName(p));
      e.style.transitionTimingFunction = AdaptList(input.TransitionEasingModes, mode => new EasingFunction(mode switch
      {
        EasingMode.Ease => UnityEngine.UIElements.EasingMode.Ease,
        EasingMode.EaseIn => UnityEngine.UIElements.EasingMode.EaseIn,
        EasingMode.EaseOut => UnityEngine.UIElements.EasingMode.EaseOut,
        EasingMode.EaseInOut => UnityEngine.UIElements.EasingMode.EaseInOut,
        EasingMode.Linear => UnityEngine.UIElements.EasingMode.Linear,
        EasingMode.EaseInSine => UnityEngine.UIElements.EasingMode.EaseInSine,
        EasingMode.EaseOutSine => UnityEngine.UIElements.EasingMode.EaseOutSine,
        EasingMode.EaseInOutSine => UnityEngine.UIElements.EasingMode.EaseInOutSine,
        EasingMode.EaseInCubic => UnityEngine.UIElements.EasingMode.EaseInCubic,
        EasingMode.EaseOutCubic => UnityEngine.UIElements.EasingMode.EaseOutCubic,
        EasingMode.EaseInOutCubic => UnityEngine.UIElements.EasingMode.EaseInOutCubic,
        EasingMode.EaseInCirc => UnityEngine.UIElements.EasingMode.EaseInCirc,
        EasingMode.EaseOutCirc => UnityEngine.UIElements.EasingMode.EaseOutCirc,
        EasingMode.EaseInOutCirc => UnityEngine.UIElements.EasingMode.EaseInOutCirc,
        EasingMode.EaseInElastic => UnityEngine.UIElements.EasingMode.EaseInElastic,
        EasingMode.EaseOutElastic => UnityEngine.UIElements.EasingMode.EaseOutElastic,
        EasingMode.EaseInOutElastic => UnityEngine.UIElements.EasingMode.EaseInOutElastic,
        EasingMode.EaseInBack => UnityEngine.UIElements.EasingMode.EaseInBack,
        EasingMode.EaseOutBack => UnityEngine.UIElements.EasingMode.EaseOutBack,
        EasingMode.EaseInOutBack => UnityEngine.UIElements.EasingMode.EaseInOutBack,
        EasingMode.EaseInBounce => UnityEngine.UIElements.EasingMode.EaseInBounce,
        EasingMode.EaseOutBounce => UnityEngine.UIElements.EasingMode.EaseOutBounce,
        EasingMode.EaseInOutBounce => UnityEngine.UIElements.EasingMode.EaseInOutBounce,
        _ => UnityEngine.UIElements.EasingMode.Ease
      }));
      e.style.translate = input.Translate is { } translate
        ? new Translate(AdaptDimensionNonNull(registry, translate.X), AdaptDimensionNonNull(registry, translate.Y),
          translate.Z)
        : new StyleTranslate(StyleKeyword.Null);
      e.style.unityBackgroundImageTintColor = AdaptColor(input.BackgroundImageTintColor);
      e.style.unityBackgroundScaleMode = input.BackgroundImageScaleMode switch
      {
        ImageScaleMode.StretchToFill => ScaleMode.StretchToFill,
        ImageScaleMode.ScaleAndCrop => ScaleMode.ScaleAndCrop,
        ImageScaleMode.ScaleToFit => ScaleMode.ScaleToFit,
        _ => new StyleEnum<ScaleMode>(StyleKeyword.Null)
      };
      e.style.unityFontDefinition = input.Font is { } font
        ? new StyleFontDefinition(registry.AssetService.GetFont(font))
        : new StyleFontDefinition(StyleKeyword.Null);
      e.style.unityFontStyleAndWeight = input.FontStyle switch
      {
        FontStyle.Normal => UnityEngine.FontStyle.Normal,
        FontStyle.Bold => UnityEngine.FontStyle.Bold,
        FontStyle.Italic => UnityEngine.FontStyle.Italic,
        FontStyle.BoldAndItalic => UnityEngine.FontStyle.BoldAndItalic,
        _ => new StyleEnum<UnityEngine.FontStyle>(StyleKeyword.Null)
      };
      e.style.unityOverflowClipBox = input.OverflowClipBox switch
      {
        OverflowClipBox.PaddingBox => UnityEngine.UIElements.OverflowClipBox.PaddingBox,
        OverflowClipBox.ContentBox => UnityEngine.UIElements.OverflowClipBox.ContentBox,
        _ => new StyleEnum<UnityEngine.UIElements.OverflowClipBox>(StyleKeyword.Null)
      };
      e.style.unityParagraphSpacing = AdaptDimension(registry, input.ParagraphSpacing);
      e.style.unitySliceTop = AdaptInt(input.ImageSlice?.Top);
      e.style.unitySliceRight = AdaptInt(input.ImageSlice?.Right);
      e.style.unitySliceBottom = AdaptInt(input.ImageSlice?.Bottom);
      e.style.unitySliceLeft = AdaptInt(input.ImageSlice?.Left);
      e.style.unityTextAlign = input.TextAlign switch
      {
        TextAlign.UpperLeft => TextAnchor.UpperLeft,
        TextAlign.UpperCenter => TextAnchor.UpperCenter,
        TextAlign.UpperRight => TextAnchor.UpperRight,
        TextAlign.MiddleLeft => TextAnchor.MiddleLeft,
        TextAlign.MiddleCenter => TextAnchor.MiddleCenter,
        TextAlign.MiddleRight => TextAnchor.MiddleRight,
        TextAlign.LowerLeft => TextAnchor.LowerLeft,
        TextAlign.LowerCenter => TextAnchor.LowerCenter,
        TextAlign.LowerRight => TextAnchor.LowerRight,
        _ => new StyleEnum<TextAnchor>(StyleKeyword.Null)
      };
      e.style.unityTextOutlineColor = AdaptColor(input.TextOutlineColor);
      e.style.unityTextOutlineWidth = AdaptFloat(input.TextOutlineWidth);
      e.style.unityTextOverflowPosition = input.TextOverflowPosition switch
      {
        TextOverflowPosition.End => UnityEngine.UIElements.TextOverflowPosition.End,
        TextOverflowPosition.Start => UnityEngine.UIElements.TextOverflowPosition.Start,
        TextOverflowPosition.Middle => UnityEngine.UIElements.TextOverflowPosition.Middle,
        _ => new StyleEnum<UnityEngine.UIElements.TextOverflowPosition>(StyleKeyword.Null)
      };
      e.style.visibility = input.Visibility switch
      {
        FlexVisibility.Visible => Visibility.Visible,
        FlexVisibility.Hidden => Visibility.Hidden,
        _ => new StyleEnum<Visibility>(StyleKeyword.Null)
      };
      e.style.whiteSpace = input.WhiteSpace switch
      {
        WhiteSpace.Normal => UnityEngine.UIElements.WhiteSpace.Normal,
        WhiteSpace.NoWrap => UnityEngine.UIElements.WhiteSpace.NoWrap,
        _ => new StyleEnum<UnityEngine.UIElements.WhiteSpace>(StyleKeyword.Null)
      };
      e.style.width = AdaptDimension(registry, input.Width);
      e.style.wordSpacing = AdaptDimension(registry, input.WordSpacing);

      if (input.BackgroundImage is { } bi)
      {
        switch (bi.BackgroundAddressCase)
        {
          case NodeBackground.BackgroundAddressOneofCase.Sprite:
            var sprite = registry.AssetService.GetSprite(bi.Sprite);
            var aspectRatio = sprite == null ? 0 : ((float)sprite.texture.width) / sprite.texture.height;

            switch (input.BackgroundImageAutoSize)
            {
              case BackgroundImageAutoSize.FromWidth:
                var height = input.Width.Clone();
                Errors.CheckState(height.Unit != DimensionUnit.Percentage, 
                  "Percentage units not supported for background image auto size");
                height.Value /= aspectRatio;
                e.style.height = AdaptDimension(registry, height);
                break;
              case BackgroundImageAutoSize.FromHeight:
                var width = input.Height.Clone();
                Errors.CheckState(width.Unit != DimensionUnit.Percentage, 
                  "Percentage units not supported for background image auto size");                
                width.Value *= aspectRatio;
                e.style.width = AdaptDimension(registry, width);                
                break;
            }
            
            e.style.backgroundImage = new StyleBackground(sprite);
            break;
          case NodeBackground.BackgroundAddressOneofCase.RenderTexture:
            var renderTexture = registry.AssetService.GetRenderTexture(registry, bi.RenderTexture);
            e.style.backgroundImage = new StyleBackground(new Background { renderTexture = renderTexture });
            break;
          case NodeBackground.BackgroundAddressOneofCase.StudioDisplay:
            registry.StudioManager.DisplayAsBackground(e, input.BackgroundImage.StudioDisplay);
            break;
        }
      }
      else
      {
        e.style.backgroundImage = new StyleBackground(StyleKeyword.Null);
      }

      e.pickingMode = input.PickingMode switch
      {
        FlexPickingMode.Unspecified => PickingMode.Position,
        FlexPickingMode.Position => PickingMode.Position,
        FlexPickingMode.Ignore => PickingMode.Ignore,
        _ => throw new ArgumentOutOfRangeException()
      };
    }
  }
}