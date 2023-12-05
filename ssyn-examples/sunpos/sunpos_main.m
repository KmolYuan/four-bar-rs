clear; clc
from = juliandate(2023, 12, 4);
into = juliandate(2024, 12, 4);
space = linspace(from, into, 100);
coords = zeros(length(space), 3);
for i = 1:length(space)
    dt = datetime(space(i), "ConvertFrom", "juliandate");
    [Sx, Sy, Sz] = sunpos(year(dt), month(dt), day(dt), 8, 23.69781, 120.960515 - 90);
    coords(i, 1) = Sx;
    coords(i, 2) = Sy;
    coords(i, 3) = Sz;
    assert(abs(Sx^2 + Sy^2 + Sz^2 - 1) < 1e-15)
end
writematrix(coords, "sunpos-taiwan.closed.csv")

figure
plot3(coords(1,1), coords(1,2), coords(1,3), 'bo')
hold on
plot3(coords(2:end-1,1), coords(2:end-1,2), coords(2:end-1,3), 'ro')
plot3(coords(end,1), coords(end,2), coords(end,3), 'bo')
xlabel("X")
ylabel("Y")
zlabel("Z")
axis equal
