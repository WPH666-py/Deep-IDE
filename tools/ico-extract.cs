using System;
using System.IO;
using System.Collections.Generic;
using System.Runtime.InteropServices;

public static class IconExtractor
{
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    static extern IntPtr LoadLibraryEx(string lpFileName, IntPtr hFile, uint dwFlags);
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern IntPtr FindResource(IntPtr hModule, int lpName, int lpType);
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern uint SizeofResource(IntPtr hModule, IntPtr hResInfo);
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern IntPtr LoadResource(IntPtr hModule, IntPtr hResInfo);
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern IntPtr LockResource(IntPtr hResData);
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool FreeLibrary(IntPtr hModule);

    public class Entry
    {
        public byte W, H;
        public ushort Planes, Bpp;
        public byte[] Blob;
    }

    public static List<Entry> Extract(string exePath)
    {
        IntPtr h = LoadLibraryEx(exePath, IntPtr.Zero, 0x2);
        if (h == IntPtr.Zero) throw new Exception("LoadLibraryEx failed: " + Marshal.GetLastWin32Error());
        try
        {
            var result = new List<Entry>();
            for (int id = 1; id <= 64; id++)
            {
                IntPtr ir = FindResource(h, id, 3);
                if (ir == IntPtr.Zero) continue;
                uint isz = SizeofResource(h, ir);
                IntPtr ip = LoadResource(h, ir);
                if (ip == IntPtr.Zero) continue;
                IntPtr idata = LockResource(ip);
                byte[] blob = new byte[isz];
                Marshal.Copy(idata, blob, 0, (int)isz);
                int biWidth = BitConverter.ToInt32(blob, 4);
                int biHeight = BitConverter.ToInt32(blob, 8);
                int biBitCount = BitConverter.ToInt16(blob, 14);
                result.Add(new Entry
                {
                    W = biWidth >= 256 ? (byte)0 : (byte)biWidth,
                    H = (biHeight / 2) >= 256 ? (byte)0 : (byte)(biHeight / 2),
                    Planes = 1,
                    Bpp = (ushort)biBitCount,
                    Blob = blob
                });
            }
            return result;
        }
        finally
        {
            FreeLibrary(h);
        }
    }

    public static void WriteIco(List<Entry> entries, string outPath)
    {
        using (var ms = new MemoryStream())
        using (var bw = new BinaryWriter(ms))
        {
            bw.Write((ushort)0); bw.Write((ushort)1); bw.Write((ushort)entries.Count);
            int offset = 6 + 16 * entries.Count;
            List<long> dataStarts = new List<long>();
            foreach (var e in entries)
            {
                bw.Write(e.W); bw.Write(e.H); bw.Write((byte)0); bw.Write((byte)0);
                bw.Write(e.Planes); bw.Write(e.Bpp);
                bw.Write((uint)e.Blob.Length); bw.Write((uint)offset);
                offset += e.Blob.Length;
                dataStarts.Add(ms.Position);
                bw.Write(new byte[e.Blob.Length]); // placeholder
            }
            for (int i = 0; i < entries.Count; i++)
            {
                long pos = ms.Position;
                ms.Position = dataStarts[i];
                bw.Write(entries[i].Blob);
                ms.Position = pos;
            }
            File.WriteAllBytes(outPath, ms.ToArray());
        }
    }
}
