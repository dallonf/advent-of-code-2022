require("scripts.puzzles.test_algo.rand")

function Draw(ctx)
  for i = 1, 100 do
    ctx.draw_rectangle(100, 100, 150, 150)
  end


  -- print(package.loaded["scripts.puzzles.test_algo.rand"])
  -- print(package.searchpath("scripts.puzzles.test_algo.rand", package.path))
  -- local keyset = {}
  -- for k, v in pairs(package.loaded["scripts.puzzles.test_algo.rand"]) do
  --   table.insert(keyset, k)
  -- end

  return Rand() .. " hi..."
end
