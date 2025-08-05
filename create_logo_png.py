#!/usr/bin/env python3
"""
Создание PNG логотипа для BSL Analyzer VSCode расширения
"""

from PIL import Image, ImageDraw, ImageFont
import math

def create_bsl_logo(size=128):
    # Создаем изображение с прозрачным фоном
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    center = size // 2
    radius = size // 2 - 8
    
    # Основной круг с градиентом (имитация)
    for i in range(radius):
        alpha = int(255 * (radius - i) / radius)
        r = int(206 - (206 - 139) * i / radius)  # CE422B -> 8B2C1B
        g = int(66 - (66 - 44) * i / radius)
        b = int(43 - (43 - 27) * i / radius)
        draw.ellipse([center-radius+i, center-radius+i, center+radius-i, center+radius-i], 
                    fill=(r, g, b, alpha))
    
    # Внутренний круг для контраста
    inner_radius = radius - 8
    draw.ellipse([center-inner_radius, center-inner_radius, center+inner_radius, center+inner_radius], 
                outline=(255, 255, 255, 50), width=1)
    
    # Левая скобка
    bracket_points_left = [
        (center-20, center-25), (center-30, center-25), (center-30, center-15),
        (center-25, center-10), (center-30, center-5), (center-30, center+5),
        (center-25, center+10), (center-30, center+15), (center-30, center+25),
        (center-20, center+25)
    ]
    
    # Правая скобка
    bracket_points_right = [
        (center+20, center-25), (center+30, center-25), (center+30, center-15),
        (center+25, center-10), (center+30, center-5), (center+30, center+5),
        (center+25, center+10), (center+30, center+15), (center+30, center+25),
        (center+20, center+25)
    ]
    
    # Рисуем скобки
    for i in range(len(bracket_points_left)-1):
        draw.line([bracket_points_left[i], bracket_points_left[i+1]], fill=(255, 255, 255, 255), width=3)
        draw.line([bracket_points_right[i], bracket_points_right[i+1]], fill=(255, 255, 255, 255), width=3)
    
    # Центральные элементы (точки типобезопасности)
    points = [
        (center, center-8),
        (center-8, center),
        (center+8, center),
        (center, center+8)
    ]
    
    for point in points:
        draw.ellipse([point[0]-3, point[1]-3, point[0]+3, point[1]+3], 
                    fill=(0, 122, 204, 255))  # Синий цвет BSL
    
    # Соединительные линии
    connections = [
        ((center, center-5), (center-5, center)),
        ((center-5, center), (center, center+5)),
        ((center, center+5), (center+5, center)),
        ((center+5, center), (center, center-5))
    ]
    
    for start, end in connections:
        draw.line([start, end], fill=(255, 255, 255, 200), width=2)
    
    # Rust gear в правом нижнем углу
    gear_center = (center + 30, center + 30)
    gear_radius = 4
    draw.ellipse([gear_center[0]-gear_radius, gear_center[1]-gear_radius,
                 gear_center[0]+gear_radius, gear_center[1]+gear_radius], 
                fill=(255, 255, 255, 230))
    
    # Gear teeth
    gear_inner = 2
    draw.ellipse([gear_center[0]-gear_inner, gear_center[1]-gear_inner,
                 gear_center[0]+gear_inner, gear_center[1]+gear_inner], 
                fill=(206, 66, 43, 200))
    
    # 1C идентификатор
    text_center = (center + 30, center - 30)
    draw.rectangle([text_center[0]-8, text_center[1]-8, text_center[0]+8, text_center[1]+8], 
                  fill=(255, 255, 255, 50))
    
    # Попробуем нарисовать текст
    try:
        font = ImageFont.truetype("arial.ttf", 10)
    except:
        font = ImageFont.load_default()
    
    draw.text(text_center, "1C", fill=(255, 255, 255, 255), font=font, anchor="mm")
    
    return img

if __name__ == "__main__":
    # Создаем логотип 128x128
    logo = create_bsl_logo(128)
    logo.save("vscode-extension/images/bsl-analyzer-logo.png", "PNG")
    print("Logo created: vscode-extension/images/bsl-analyzer-logo.png")
    
    # Создаем дополнительные размеры
    sizes = [32, 64, 256]
    for size in sizes:
        logo_resized = create_bsl_logo(size)
        logo_resized.save(f"vscode-extension/images/bsl-analyzer-logo-{size}.png", "PNG")
        print(f"Logo {size}x{size} created")