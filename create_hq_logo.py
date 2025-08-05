#!/usr/bin/env python3
"""
Создание высококачественного PNG логотипа для BSL Analyzer VSCode расширения
с улучшенным anti-aliasing и сглаживанием
"""

from PIL import Image, ImageDraw, ImageFont, ImageFilter
import math

def create_smooth_circle(draw, center, radius, color, width=0):
    """Создание сглаженного круга"""
    x, y = center
    if width == 0:  # Заливка
        draw.ellipse([x-radius, y-radius, x+radius, y+radius], fill=color)
    else:  # Обводка
        draw.ellipse([x-radius, y-radius, x+radius, y+radius], outline=color, width=width)

def create_smooth_line(draw, start, end, color, width=3):
    """Создание сглаженной линии"""
    draw.line([start, end], fill=color, width=width)

def create_hq_bsl_logo(size=128):
    # Создаем изображение с увеличенным разрешением для лучшего качества
    scale = 4  # Увеличиваем в 4 раза для super-sampling
    work_size = size * scale
    img = Image.new('RGBA', (work_size, work_size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    center = work_size // 2
    radius = work_size // 2 - 32  # Больше отступ для четких краев
    
    # Основной круг с градиентом (многослойный для плавности)
    gradient_steps = 50
    for i in range(gradient_steps):
        alpha = int(255 * (gradient_steps - i) / gradient_steps * 0.9)  # Немного прозрачнее
        progress = i / gradient_steps
        
        # Цветовой градиент от CE422B к 8B2C1B
        r = int(206 - (206 - 139) * progress)
        g = int(66 - (66 - 44) * progress) 
        b = int(43 - (43 - 27) * progress)
        
        current_radius = radius - i * radius // gradient_steps
        if current_radius > 0:
            create_smooth_circle(draw, (center, center), current_radius, (r, g, b, alpha))
    
    # Внутренний контур для четкости
    inner_radius = radius - 32
    create_smooth_circle(draw, (center, center), inner_radius, (255, 255, 255, 40), width=4)
    
    # Левая скобка - более плавные кривые
    bracket_width = 12
    bracket_color = (255, 255, 255, 255)
    
    # Левая скобка - составные части для плавности
    left_points = [
        (center - 80, center - 100),  # Верх
        (center - 120, center - 100), 
        (center - 120, center - 60),
        (center - 100, center - 40),  # Первый изгиб
        (center - 120, center - 20),
        (center - 120, center + 20),
        (center - 100, center + 40),  # Второй изгиб
        (center - 120, center + 60),
        (center - 120, center + 100),
        (center - 80, center + 100)   # Низ
    ]
    
    # Рисуем левую скобку сегментами для плавности
    for i in range(len(left_points) - 1):
        create_smooth_line(draw, left_points[i], left_points[i+1], bracket_color, bracket_width)
    
    # Правая скобка - зеркальное отражение
    right_points = [(center + (center - x), y) for x, y in left_points]
    for i in range(len(right_points) - 1):
        create_smooth_line(draw, right_points[i], right_points[i+1], bracket_color, bracket_width)
    
    # Центральные элементы - точки типобезопасности
    dot_radius = 12
    dot_color = (0, 122, 204, 255)  # Синий BSL
    shadow_color = (0, 100, 180, 100)  # Тень
    
    points = [
        (center, center - 32),      # Верх
        (center - 32, center),      # Лево
        (center + 32, center),      # Право
        (center, center + 32)       # Низ
    ]
    
    # Рисуем точки с тенями
    for point in points:
        # Тень
        create_smooth_circle(draw, (point[0] + 2, point[1] + 2), dot_radius + 1, shadow_color)
        # Основная точка
        create_smooth_circle(draw, point, dot_radius, dot_color)
        # Блик
        create_smooth_circle(draw, (point[0] - 3, point[1] - 3), dot_radius // 3, (255, 255, 255, 150))
    
    # Соединительные линии с градиентом прозрачности
    line_color = (255, 255, 255, 200)
    line_width = 6
    
    connections = [
        ((center, center - 20), (center - 20, center)),
        ((center - 20, center), (center, center + 20)),
        ((center, center + 20), (center + 20, center)),
        ((center + 20, center), (center, center - 20))
    ]
    
    for start, end in connections:
        create_smooth_line(draw, start, end, line_color, line_width)
    
    # Rust gear в правом нижнем углу - более детальный
    gear_center = (center + 120, center + 120)
    gear_outer = 16
    gear_inner = 8
    
    # Тень gear
    create_smooth_circle(draw, (gear_center[0] + 2, gear_center[1] + 2), gear_outer + 2, (0, 0, 0, 100))
    # Основа gear
    create_smooth_circle(draw, gear_center, gear_outer, (255, 255, 255, 230))
    # Внутренняя часть
    create_smooth_circle(draw, gear_center, gear_inner, (206, 66, 43, 200))
    # Центр
    create_smooth_circle(draw, gear_center, gear_inner // 2, (255, 255, 255, 255))
    
    # 1C идентификатор - улучшенный дизайн
    text_center = (center + 120, center - 120)
    rect_size = 32
    
    # Фон для текста
    draw.rounded_rectangle([
        text_center[0] - rect_size, text_center[1] - rect_size,
        text_center[0] + rect_size, text_center[1] + rect_size
    ], radius=8, fill=(255, 255, 255, 60), outline=(255, 255, 255, 120), width=2)
    
    # Масштабируем размер шрифта
    font_size = 40
    try:
        font = ImageFont.truetype("arial.ttf", font_size)
    except:
        font = ImageFont.load_default()
    
    # Рисуем текст с тенью
    text = "1C"
    # Тень
    draw.text((text_center[0] + 2, text_center[1] + 2), text, fill=(0, 0, 0, 100), font=font, anchor="mm")
    # Основной текст
    draw.text(text_center, text, fill=(255, 255, 255, 255), font=font, anchor="mm")
    
    # Уменьшаем до нужного размера с высококачественной интерполяцией
    img = img.resize((size, size), Image.Resampling.LANCZOS)
    
    # Применяем легкую резкость для четкости
    img = img.filter(ImageFilter.UnsharpMask(radius=0.5, percent=120, threshold=2))
    
    return img

def create_optimized_icons():
    """Создание оптимизированных иконок для разных размеров"""
    
    # VSCode использует эти размеры
    sizes = {
        16: "small",      # Статусная строка
        24: "medium",     # Панель активности  
        32: "large",      # Список расширений
        48: "xlarge",     # Детали расширения
        128: "icon"       # Основная иконка
    }
    
    icons_created = []
    
    for size, suffix in sizes.items():
        print(f"Creating {size}x{size} icon...")
        
        # Для маленьких размеров упрощаем дизайн
        if size <= 24:
            logo = create_simplified_logo(size)
        else:
            logo = create_hq_bsl_logo(size)
        
        filename = f"vscode-extension/images/bsl-analyzer-{suffix}.png"
        logo.save(filename, "PNG", optimize=True)
        icons_created.append(f"{size}x{size} -> {filename}")
        print(f"  Saved: {filename}")
    
    return icons_created

def create_simplified_logo(size):
    """Упрощенная версия для маленьких размеров"""
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    center = size // 2
    radius = size // 2 - 2
    
    # Простой круг
    draw.ellipse([2, 2, size-2, size-2], fill=(206, 66, 43, 255))
    
    # Простые скобки
    bracket_width = max(1, size // 16)
    draw.rectangle([center//2, center//2, center//2 + bracket_width, center + center//2], fill=(255, 255, 255, 255))
    draw.rectangle([center + center//2, center//2, center + center//2 + bracket_width, center + center//2], fill=(255, 255, 255, 255))
    
    # Центральная точка
    dot_size = max(1, size // 8)
    draw.ellipse([center-dot_size, center-dot_size, center+dot_size, center+dot_size], fill=(0, 122, 204, 255))
    
    return img

if __name__ == "__main__":
    print("Creating high-quality BSL Analyzer icons...")
    
    # Создаем основную иконку
    main_logo = create_hq_bsl_logo(128)
    main_logo.save("vscode-extension/images/bsl-analyzer-logo.png", "PNG", optimize=True)
    print("Main logo created: bsl-analyzer-logo.png")
    
    # Создаем оптимизированные версии
    icons = create_optimized_icons()
    
    print(f"\nSuccessfully created {len(icons) + 1} high-quality icons:")
    print("  - bsl-analyzer-logo.png (128x128)")
    for icon in icons:
        print(f"  - {icon}")
    
    print("\nFeatures:")
    print("  + High-quality anti-aliasing")
    print("  + Smooth gradients and shadows") 
    print("  + Optimized for different sizes")
    print("  + Professional appearance in VSCode")