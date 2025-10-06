#include <efi.h>
#include <efilib.h>

#define KERNEL_LOAD_ADDRESS 0x200000

EFI_STATUS
EFIAPI
efi_main(EFI_HANDLE ImageHandle, EFI_SYSTEM_TABLE *SystemTable)
{
    EFI_STATUS Status;
    EFI_LOADED_IMAGE_PROTOCOL *LoadedImage;
    EFI_FILE_PROTOCOL *RootDir;
    EFI_FILE_PROTOCOL *KernelFile;
    EFI_FILE_INFO *FileInfo;
    UINTN FileInfoSize;
    UINTN KernelSize;
    VOID *KernelBuffer;
    UINTN MapSize, MapKey, DescriptorSize;
    UINT32 DescriptorVersion;
    EFI_MEMORY_DESCRIPTOR *MemoryMap;
    
    InitializeLib(ImageHandle, SystemTable);
    Print(L"NexusOS UEFI Bootloader v1.0\r\n");
    
    // Get loaded image protocol
    Status = uefi_call_wrapper(BS->HandleProtocol, 3,
                               ImageHandle,
                               &gEfiLoadedImageProtocolGuid,
                               (VOID**)&LoadedImage);
    if (EFI_ERROR(Status)) {
        Print(L"Failed to get loaded image protocol\r\n");
        return Status;
    }
    
    // Get root directory
    EFI_SIMPLE_FILE_SYSTEM_PROTOCOL *FileSystem;
    Status = uefi_call_wrapper(BS->HandleProtocol, 3,
                               LoadedImage->DeviceHandle,
                               &gEfiSimpleFileSystemProtocolGuid,
                               (VOID**)&FileSystem);
    if (EFI_ERROR(Status)) {
        Print(L"Failed to get file system protocol\r\n");
        return Status;
    }
    
    Status = uefi_call_wrapper(FileSystem->OpenVolume, 2,
                               FileSystem, &RootDir);
    if (EFI_ERROR(Status)) {
        Print(L"Failed to open root directory\r\n");
        return Status;
    }
    
    // Open kernel file
    Status = uefi_call_wrapper(RootDir->Open, 5,
                               RootDir, &KernelFile,
                               L"nexus-kernel.bin",
                               EFI_FILE_MODE_READ,
                               0);
    if (EFI_ERROR(Status)) {
        Print(L"Failed to open kernel file\r\n");
        return Status;
    }
    
    // Get kernel file size
    FileInfoSize = sizeof(EFI_FILE_INFO) + 512;
    FileInfo = AllocatePool(FileInfoSize);
    Status = uefi_call_wrapper(KernelFile->GetInfo, 4,
                               KernelFile,
                               &gEfiFileInfoGuid,
                               &FileInfoSize,
                               FileInfo);
    if (EFI_ERROR(Status)) {
        Print(L"Failed to get kernel file info\r\n");
        return Status;
    }
    
    KernelSize = FileInfo->FileSize;
    FreePool(FileInfo);
    
    Print(L"Loading kernel (%lu bytes)...\r\n", KernelSize);
    
    // Allocate memory for kernel
    Status = uefi_call_wrapper(BS->AllocatePages, 4,
                               AllocateAddress,
                               EfiLoaderData,
                               (KernelSize / 4096) + 1,
                               (EFI_PHYSICAL_ADDRESS*)&KernelBuffer);
    if (EFI_ERROR(Status)) {
        Print(L"Failed to allocate kernel memory\r\n");
        return Status;
    }
    
    // Read kernel into memory
    Status = uefi_call_wrapper(KernelFile->Read, 3,
                               KernelFile,
                               &KernelSize,
                               KernelBuffer);
    if (EFI_ERROR(Status)) {
        Print(L"Failed to read kernel\r\n");
        return Status;
    }
    
    uefi_call_wrapper(KernelFile->Close, 1, KernelFile);
    uefi_call_wrapper(RootDir->Close, 1, RootDir);
    
    Print(L"Kernel loaded at 0x%lx\r\n", (UINT64)KernelBuffer);
    
    // Get memory map
    MapSize = 0;
    Status = uefi_call_wrapper(BS->GetMemoryMap, 5,
                               &MapSize, NULL, &MapKey,
                               &DescriptorSize, &DescriptorVersion);
    
    MapSize += 2 * DescriptorSize;
    MemoryMap = AllocatePool(MapSize);
    
    Status = uefi_call_wrapper(BS->GetMemoryMap, 5,
                               &MapSize, MemoryMap, &MapKey,
                               &DescriptorSize, &DescriptorVersion);
    if (EFI_ERROR(Status)) {
        Print(L"Failed to get memory map\r\n");
        return Status;
    }
    
    // Exit boot services
    Print(L"Exiting boot services...\r\n");
    Status = uefi_call_wrapper(BS->ExitBootServices, 2,
                               ImageHandle, MapKey);
    if (EFI_ERROR(Status)) {
        Print(L"Failed to exit boot services\r\n");
        return Status;
    }
    
    // Jump to kernel
    void (*kernel_entry)(void) = (void(*)(void))KERNEL_LOAD_ADDRESS;
    kernel_entry();
    
    // Should never reach here
    return EFI_SUCCESS;
}