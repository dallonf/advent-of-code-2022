local rand = require("scripts.puzzles.test_algo.rand")
local inspect = require("scripts.vendor.inspect")

function ProcessEvent(event)
  print(event.type)
  print(inspect.inspect(event))
end

function Draw(ctx)
  for i = 1, 10 do
    ctx.draw_rectangle(100 + rand.rand() * 400, 100 + rand.rand() * 400, 150, 150)
  end

  return rand.rand() .. " hi!"
end
