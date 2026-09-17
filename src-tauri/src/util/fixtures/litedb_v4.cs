// Writes src/util/fixtures/litedb_v4.db with LiteDB 4.1.4, the version Playnite uses.
// Run from a console project referencing the LiteDB 4.1.4 package: dotnet run -- <output path>
// Index pages are randomized, so bytes differ between runs; the documents do not.
using System;
using System.Collections.Generic;
using System.IO;
using LiteDB;

public class Item
{
    public Guid Id { get; set; }
    public string Name { get; set; }
    public bool IsInstalled { get; set; }
    public string Description { get; set; }
    public int PlayCount { get; set; }
    public long Playtime { get; set; }
    public double Score { get; set; }
    public decimal Price { get; set; }
    public DateTime? Added { get; set; }
    public List<string> Tags { get; set; }
    public Dictionary<string, int> Stats { get; set; }
    public byte[] Blob { get; set; }
    public ObjectId Ref { get; set; }
}

public static class Program
{
    static Guid G(int n) => new Guid(n, 0x1122, 0x3344, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc);

    public static void Main(string[] args)
    {
        var path = args[0];
        if (File.Exists(path)) File.Delete(path);
        var mapper = new BsonMapper();
        mapper.Entity<Item>().Id(i => i.Id, false);
        using (var db = new LiteDatabase($"Filename={path};Mode=Exclusive;Cache Size=0", mapper))
        {
            var col = db.GetCollection<Item>("items");
            col.Insert(new Item
            {
                Id = G(1), Name = "Typed", IsInstalled = true, PlayCount = 42, Playtime = 1L << 40,
                Score = 9.5, Price = 19.99m, Added = new DateTime(2020, 1, 2, 3, 4, 5, DateTimeKind.Utc),
                Tags = new List<string> { "a", "b" }, Stats = new Dictionary<string, int> { ["wins"] = 3 },
                Blob = new byte[] { 1, 2, 3 }, Ref = new ObjectId("5f0000000000000000000001"),
            });
            col.Insert(new Item { Id = G(2), Name = "Not installed" });
            // Larger than a data page, so it spills into extend pages
            col.Insert(new Item { Id = G(3), Name = "Large", IsInstalled = true, Description = new string('x', 10000) });
            col.Insert(new Item { Id = G(4), Name = "Deleted" });
            col.Delete(G(4));
            // Grows past its block on update, so it moves to extend pages
            col.Insert(new Item { Id = G(5), Name = "Grown" });
            var grown = col.FindById(G(5));
            grown.Description = new string('y', 5000);
            col.Update(grown);
            col.Insert(new Item { Id = G(6), Name = "Ünïcödé 名前" });
            // The mapper leaves out nulls, so write those (and the marker values) directly, in a second collection
            db.GetCollection("raw").Insert(new BsonDocument
            {
                ["_id"] = 7,
                ["Name"] = "Raw",
                ["Nothing"] = BsonValue.Null,
                ["Min"] = BsonValue.MinValue,
                ["Max"] = BsonValue.MaxValue,
            });
        }
        Console.WriteLine($"wrote {new FileInfo(path).Length} bytes");
    }
}
