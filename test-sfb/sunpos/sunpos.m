function [Sx, Sy, Sz] = sunpos(inyear, inmon, inday, gmtime, xlat, xlon)
% SUNPOS Solar Geometry using subsolar point and atan2.
%   [Sx, Sy, Sz] = SUNPOS(inyear, inmon, inday, gmtime, xlat, xlon)
%
% Input Variables
%   inyear: 4-digit year, e.g., 1998, 2020.
%   inmon: month, in the range of 1-12.
%   inday: day, in the range of 1-28/29/30/31.
%   gmtime: GMT in decimal hour, e.g., 15.2167.
%   xlat: latitude in decimal degree, positive in Northern Hemisphere.
%   xlon: longitude in decimal degree, positive for East longitude.
%
% Output Variables
%   Sx, Sy, Sz: The coordinate of the sun.
nday = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
julday = zeros(1, 12);
rpd = acos(-1) / 180;
if (mod(inyear, 100) ~= 0 && mod(inyear, 4) == 0) || (mod(inyear, 100) == 0 && mod(inyear, 400) == 0)
    nday(2) = 29;
else
    nday(2) = 28;
end

for i = 1:12
    if i == 1
        julday(i) = nday(i);
    else
        julday(i) = julday(i - 1) + nday(i);
    end
end

dyear = inyear - 2000;
if inmon == 1
    dayofyr = inday;
else
    dayofyr = julday(inmon - 1) + inday;
end
xleap = int32(floor(real(dyear) / 4));
if dyear > 0 && mod(dyear, 4) ~= 0
    xleap = xleap + 1;
end
xleap = double(xleap);

n = -1.5 + dyear * 365 + xleap + dayofyr + gmtime / 24;
L = mod(280.460 + 0.9856474 * n, 360);
g = mod(357.528 + 0.9856003 * n, 360);
lambda = mod(L + 1.915 * sin(g * rpd) + 0.020 * sin(2 * g * rpd), 360);
epsilon = 23.439 - 0.0000004 * n;
alpha = mod(atan2(cos(epsilon * rpd) * sin(lambda * rpd), cos(lambda * rpd)) / rpd, 360);
delta = asin(sin(epsilon * rpd) * sin(lambda * rpd)) / rpd;
% R = 1.00014 - 0.01671 * cos(g * rpd) - 0.00014 * cos(2 * g * rpd);
EoT = mod((L - alpha) + 180, 360) - 180;

sunlat = delta;
sunlon = -15 * (gmtime - 12 + EoT * 4 / 60);
PHIo = xlat * rpd;
PHIs = sunlat * rpd;
LAMo = xlon * rpd;
LAMs = sunlon * rpd;
Sx = cos(PHIs) * sin(LAMs - LAMo);
Sy = cos(PHIo) * sin(PHIs) - sin(PHIo) * cos(PHIs) * cos(LAMs - LAMo);
Sz = sin(PHIo) * sin(PHIs) + cos(PHIo) * cos(PHIs) * cos(LAMs - LAMo);

% solarz = acos(Sz) / rpd;
% azi = atan2(-Sx, -Sy) / rpd;
end
