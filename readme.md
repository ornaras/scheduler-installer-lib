# Библиотека для установки SkatWorkerAPI

## Методы присутствующие в библиотеке

- bool is_installed() - возвращает `true` если SkatWorkerAPI найден в системе, иначе `false`
- int install() - скачивание и тихая установка SkatWorkerAPI с зависимостями; возвращает код ошибки
- int install(bool is_slient) - вариация метода install с настройкой режимом показа UI

## Поддерживаемые ОС

- Windows 10 (x86/amd64)
- Windows 11 (amd64)

## Пример внедрения в .NET проект

```csharp
using System.Runtime.InteropServices;

public static class Program
{
    static void Main()
    {
        if (SkatWorker.IsInstalled())
            Console.WriteLine("Планировщик уже установлен");
        else {
            var code = SkatWorker.Install();
            if(code == 0)
                Console.WriteLine("Планировщик успешно установлен");
            else
                Console.WriteLine($"При установке планировщика произошла ошибка (Код: {code}).");
        }
    }  
}

internal static class SkatWorker
{
    private const string DllName = "skatworker_installation.dll";

    [DllImport(DllName, EntryPoint = "install")]
    public static extern int Install();
    
    [DllImport(DllName, EntryPoint = "is_installed")]
    public static extern bool IsInstalled();
}
```
