π=3.141592653589793
norm_dist(x,μ,σ)=1/(σ*sqrt(2*π))*exp(-.5*((x-μ)/σ)**2)

## Provide these from outside
# dist_name=
# μ_input=
# σ_input=
# xmin=
# xmax=
# xdiff=

set xrange[xmin:xmax]
set xtics xdiff

set multiplot layout 1,2
set key top right

plot norm_dist(x,μ_input,σ_input) title dist_name

set key bottom right

plot norm((x-μ_input)/σ_input) title dist_name." (cdf)"

unset multiplot