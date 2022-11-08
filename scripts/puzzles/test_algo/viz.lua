require("scripts.puzzles.test_algo.rand")
-- local packages = {}
-- for k, v in pairs(package.loaded) do
--   local path = package.searchpath(k, package.path)
--   if path ~= nil then
--     table.insert(packages, path)
--   else
--     table.insert(packages, "nil (" .. k ..")")
--   end
-- end

function Draw(ctx)
  for i = 1, 100 do
    ctx.draw_rectangle(100, 100, 150, 150)
  end

  -- print(package.loaded["scripts.puzzles.test_algo.rand"])
  -- print(package.searchpath("scripts.puzzles.test_algo.rand", package.path))


  -- return table.concat(packages, "\n")

  return Rand() .. " hi!"
  -- return "wat..."
end
